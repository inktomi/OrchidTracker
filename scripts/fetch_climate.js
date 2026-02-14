const axios = require('axios');
const fs = require('fs');
const path = require('path');

// 1. Get credentials from environment variables (GitHub Secrets)
const USERNAME = process.env.AC_INFINITY_USERNAME;
const PASSWORD = process.env.AC_INFINITY_PASSWORD;

if (!USERNAME || !PASSWORD) {
  console.error("Please provide AC_INFINITY_USERNAME and AC_INFINITY_PASSWORD.");
  process.exit(1);
}

// 2. Constants for AC Infinity Cloud API (Reverse Engineered)
const API_BASE = "http://www.acinfinityserver.com/api";

async function run() {
  try {
    // A. Login to get token
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

    // B. Get Device List (Sensors)
    // This endpoint returns all devices and their current sensor data (temp, humid, vpd)
    const devicesResp = await axios.post(`${API_BASE}/dev/getDevInfoListAll`, {
      userId: loginResp.data.data.userId // sometimes needed, sometimes not
    }, {
      headers: { token }
    });

    if (devicesResp.data.code !== 200) {
      throw new Error(`Failed to fetch devices: ${devicesResp.data.msg}`);
    }

    const devices = devicesResp.data.data;
    console.log(`Found ${devices.length} devices.`);

    // C. Extract Relevant Data (Orchidarium Sensors)
    // Filter for devices that have sensors (temp/humidity/vpd)
    // We prioritize controllers (Controller 69 Pro, Terraform, etc.)
    const climateData = devices.map(dev => {
      // Basic info
      let info = {
        name: dev.devName,
        type: dev.devType,
        online: dev.online === 1,
        updated: new Date().toISOString()
      };

      // Sensor data (Temp/Humid/VPD usually in sensors array or direct properties)
      // Based on community docs, key properties are often:
      // temperature, humidity, vpd, trend, etc.
      // Sometimes they are nested in `sensors` array for multi-port controllers.
      
      // Attempt to find sensor data in common locations
      if (dev.sensors && dev.sensors.length > 0) {
        // Multi-port controller (e.g., 69 Pro)
        // Usually port 1 is the main sensor probe if attached
        const sensor = dev.sensors.find(s => s.sensorType > 0) || dev.sensors[0]; 
        // Note: sensorType 0 often means no sensor attached.
        
        info.temperature = sensor.temperature ? (sensor.temperature / 100).toFixed(1) : null; // Often stored as int * 100
        info.humidity = sensor.humidity ? (sensor.humidity / 100).toFixed(1) : null;
        info.vpd = sensor.vpd ? (sensor.vpd / 100).toFixed(2) : null;
      } else {
        // Single device (e.g., specific humidifier might report directly)
        info.temperature = dev.temperature ? (dev.temperature / 100).toFixed(1) : null;
        info.humidity = dev.humidity ? (dev.humidity / 100).toFixed(1) : null;
        info.vpd = dev.vpd ? (dev.vpd / 100).toFixed(2) : null;
      }
      
      return info;
    }).filter(d => d.temperature != null); // Only keep devices reporting climate data

    if (climateData.length === 0) {
      console.warn("No climate sensors found.");
    }

    // D. Save to JSON
    const dataPath = path.join(__dirname, '../src/data/climate.json');
    fs.writeFileSync(dataPath, JSON.stringify(climateData, null, 2));
    console.log(`Successfully updated climate data at ${dataPath}`);

  } catch (error) {
    console.error("Error:", error.message);
    if (error.response) {
      console.error("Response data:", error.response.data);
    }
    process.exit(1);
  }
}

run();
