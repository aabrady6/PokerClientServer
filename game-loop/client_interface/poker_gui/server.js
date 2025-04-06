const express = require("express");
const cors = require("cors");
const fetch = require("node-fetch");  
const os = require("os");
const app = express();

app.use(cors());
app.use(express.static("public"));
app.use(express.json());

require('dotenv').config({ path: './.env' });
const RUST_SERVER_IP = process.env.VITE_RUST_SERVER_IP || "localhost";
const RUST_SERVER_PORT = process.env.VITE_RUST_SERVER_PORT || "8080";
const NODE_SERVER_IP = process.env.VITE_NODE_SERVER_IP || "localhost";
const NODE_SERVER_PORT = process.env.VITE_NODE_SERVER_PORT || "3000";
const RUST_SERVER_URL = `http://${RUST_SERVER_IP}:${RUST_SERVER_PORT}`;

console.log("Loaded ENV Variables:");
console.log("RUST_SERVER_IP:", process.env.VITE_RUST_SERVER_IP);
console.log("RUST_SERVER_PORT:", process.env.VITE_RUST_SERVER_PORT);

function getLocalIp() {
  const interfaces = os.networkInterfaces();
  for (const name in interfaces) {
    for (const iface of interfaces[name]) {
      if (iface.family === "IPv4" && !iface.internal) {
        return iface.address;
      }
    }
  }
  return "localhost";  
}

// POST login route
app.post("/login/:playerName", async (req, res) => {
  const { playerName } = req.params;
  const { password } = req.body;

  if (!password) {
    return res.status(400).json({ error: "Password is required" });
  }

  try {
    const response = await fetch(`${RUST_SERVER_URL}/login/${playerName}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",  
      },
      body: JSON.stringify({
        type: "UserLogin",
        username: playerName,
        password: password,
      }),
    });
    
    if (!response.ok) {
      const error = await response.text();
      return res.status(response.status).json({ error });
    }
    
    const data = await response.json();
    res.json(data);
  } catch (error) {
    console.error("Login error:", error);
    res.status(500).json({ error: "Login failed" });
  }
});

// POST register route
app.post("/register/:playerName", async (req, res) => {
  const { playerName } = req.params;
  const { password } = req.body;

  if (!password) {
    return res.status(400).json({ error: "Password is required" });
  }

  try {
    const response = await fetch(`${RUST_SERVER_URL}/register/${playerName}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",  
      },
      body: JSON.stringify({
        type: "UserRegistration",
        username: playerName,
        password: password,
      }),
    });
    
    if (!response.ok) {
      const error = await response.text();
      return res.status(response.status).json({ error });
    }
    
    const data = await response.json();
    res.json(data);
  } catch (error) {
    console.error("Registration error:", error);
    res.status(500).json({ error: "Registration failed" });
  }
});

// GET stats route
app.get("/stats", async (req, res) => {
  const { type, stats_menu_type, selected_option } = req.query;
  
  try {
    const response = await fetch(`${RUST_SERVER_URL}/stats?type=${type}&stats_menu_type=${stats_menu_type}&selected_option=${selected_option}`, {
      method: "GET",
    });

    if (!response.ok) {
      const error = await response.text();
      return res.status(response.status).json({ error });
    }

    const data = await response.json();
    res.json(data);
  } catch (error) {
    console.error("Get stats error:", error);
    res.status(500).json({ error: "Get stats failed" });
  }
});

// GET start game route
app.get("/startgame", async (req, res) => {
  try {
    const response = await fetch(`${RUST_SERVER_URL}/startgame`, {
      method: "GET",
    });

    if (!response.ok) {
      const error = await response.text();
      return res.status(response.status).json({ error });
    }

    const data = await response.json();
    res.json(data);
  } catch (error) {
    console.error("Start game error:", error);
    res.status(500).json({ error: "Start game failed" });
  }
});

app.listen(NODE_SERVER_PORT, "0.0.0.0", () => {
  console.log(`Node server running at http://${getLocalIp()}:${NODE_SERVER_PORT}`);
});

