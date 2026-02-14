const { GoogleGenerativeAI } = require("@google/generative-ai");
const fs = require('fs');
const path = require('path');

// 1. Get species name from argument
const speciesName = process.argv[2];
if (!speciesName) {
  console.error("Please provide a species name.");
  process.exit(1);
}

// 2. Setup Gemini
const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);
const model = genAI.getGenerativeModel({ model: "gemini-1.5-flash" });

// 3. Prompt
const prompt = `
I need information about the orchid species "${speciesName}" for my care tracker.
Please provide the data in strict JSON format matching this schema:
{
  "name": "string (Common name or species name)",
  "species": "string (Scientific name)",
  "water_frequency_days": number (integer estimate),
  "light_requirement": "string (One of: 'Low', 'Medium', 'High')",
  "notes": "string (Brief care notes, max 200 chars)",
  "placement": "string (One of: 'Low', 'Medium', 'High' - suggest default)",
  "light_lux": "string (e.g. '10,000-20,000')",
  "temperature_range": "string (e.g. '15-25°C')"
}
Only return the JSON object, no markdown formatting.
`;

async function run() {
  try {
    const result = await model.generateContent(prompt);
    const response = await result.response;
    let text = response.text();
    
    // Cleanup markdown if present
    text = text.replace(/```json/g, '').replace(/```/g, '').trim();
    
    const newOrchid = JSON.parse(text);
    newOrchid.id = Date.now(); // Simple ID generation

    // 4. Update JSON file
    const dataPath = path.join(__dirname, '../src/data/orchids.json');
    let orchids = [];
    if (fs.existsSync(dataPath)) {
      const fileContent = fs.readFileSync(dataPath, 'utf8');
      orchids = JSON.parse(fileContent);
    }

    orchids.push(newOrchid);

    fs.writeFileSync(dataPath, JSON.stringify(orchids, null, 2));
    console.log(`Successfully added ${speciesName}`);

  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  }
}

run();
