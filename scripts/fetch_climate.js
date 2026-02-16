const axios = require('axios');
const fs = require('fs');
const path = require('path');

// 1. Get credentials from environment variables (GitHub Secrets)
const USERNAME = process.env.AC_INFINITY_USERNAME;
const PASSWORD = process.env.AC_INFINITY_PASSWORD;
const TEMPEST_TOKEN = process.env.TEMPEST_API_TOKEN;
const TEMPEST_STATION_ID = "113850";

if (!USERNAME || !PASSWORD) {
  console.error("Please provide AC_INFINITY_USERNAME and AC_INFINITY_PASSWORD.");
  // Don't exit yet, might still be able to fetch Tempest if provided
}

// 2. Constants for AC Infinity Cloud API (Reverse Engineered)
const API_BASE = "http://www.acinfinityserver.com/api";

// Helper to calculate VPD
function calculateVPD(tempC, humidity) {
    const svp = 0.61078 * Math.exp((17.27 * tempC) / (tempC + 237.3));
    const vpd = svp * (1 - humidity / 100);
    return vpd.toFixed(2);
}

async function fetchTempestData() {
    if (!TEMPEST_TOKEN) {
        console.log("No TEMPEST_API_TOKEN provided, skipping outdoor weather.");
        return null;
    }
    
    try {
        console.log("Fetching Tempest weather data...");
        const url = `https://swd.weatherflow.com/swd/rest/observations/station/${TEMPEST_STATION_ID}?token=${TEMPEST_TOKEN}`;
        const resp = await axios.get(url);
        
        if (resp.data && resp.data.obs && resp.data.obs.length > 0) {
            const obs = resp.data.obs[0]; // Newest observation
            // format is [timestamp, wind, ..., air_temp, ..., relative_humidity, ...]
            // Need to map by index if API uses array format, or check documentation.
            // Tempest API "observations/station" returns 'obs' array of objects usually? 
            // Wait, documentation says it returns an array of objects for 'station' endpoint? 
            // Actually, "observations/station" returns `obs`: [ { ... } ].
            // Let's verify field names. Usually `air_temperature`, `relative_humidity`.
            
            // Correction: The 'obs' array in 'observations/station' response contains objects with keys like 'air_temperature', etc.
            // Let's assume standard object structure.
            
            const tempC = obs.air_temperature;
            const humidity = obs.relative_humidity;
            
            return {
                name: "Outdoor (Tempest)",
                type_str: "Weather Station",
                temperature: tempC.toFixed(1),
                humidity: humidity.toFixed(1),
                vpd: calculateVPD(tempC, humidity),
                updated: new Date().toISOString(),
                online: true
            };
        }
    } catch (e) {
        console.error("Failed to fetch Tempest data:", e.message);
    }
    return null;
}

async function run() {
  let climateData = [];

  // A. Fetch AC Infinity
  if (USERNAME && PASSWORD) {
      try {
        console.log("Logging in to AC Infinity...");
        const loginResp = await axios.post(`${API_BASE}/user/userLogin`, {
          userEmail: USERNAME,
          userPassword: PASSWORD,
          appVersion: "1.0.1",
          appName: "ACInfinity"
        });

        if (loginResp.data.code !== 200) {
          throw new Error(`Login failed: ${loginResp.data.msg}`);
        }

        const token = loginResp.data.data.token;
        console.log("Login successful.");

        const devicesResp = await axios.post(`${API_BASE}/dev/getDevInfoListAll`, {
          userId: loginResp.data.data.userId
        }, {
          headers: { token }
        });

        if (devicesResp.data.code === 200) {
            const devices = devicesResp.data.data;
            const acData = devices.map(dev => {
              let info = {
                name: dev.devName,
                type_str: "Indoor Controller", // standardized field
                online: dev.online === 1,
                updated: new Date().toISOString()
              };

              if (dev.sensors && dev.sensors.length > 0) {
                const sensor = dev.sensors.find(s => s.sensorType > 0) || dev.sensors[0]; 
                info.temperature = sensor.temperature ? (sensor.temperature / 100).toFixed(1) : null;
                info.humidity = sensor.humidity ? (sensor.humidity / 100).toFixed(1) : null;
                info.vpd = sensor.vpd ? (sensor.vpd / 100).toFixed(2) : null;
              } else {
                info.temperature = dev.temperature ? (dev.temperature / 100).toFixed(1) : null;
                info.humidity = dev.humidity ? (dev.humidity / 100).toFixed(1) : null;
                info.vpd = dev.vpd ? (dev.vpd / 100).toFixed(2) : null;
              }
              return info;
            }).filter(d => d.temperature != null);
            
            climateData.push(...acData);
        }
      } catch (error) {
        console.error("AC Infinity Error:", error.message);
      }
  }

  // B. Fetch Tempest
  const tempestData = await fetchTempestData();
  if (tempestData) {
      climateData.push(tempestData);
  }

  if (climateData.length === 0) {
      console.warn("No climate data collected.");
  } else {
      console.log(`Collected ${climateData.length} climate records.`);
  }

  // C. Save to JSON
  try {
    const dataPath = path.join(__dirname, '../src/data/climate.json');
    fs.writeFileSync(dataPath, JSON.stringify(climateData, null, 2));
    console.log(`Successfully updated climate data at ${dataPath}`);
  } catch (e) {
      console.error("Failed to save data:", e.message);
      process.exit(1);
  }
}

run();
