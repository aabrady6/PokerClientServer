const express = require("express");
const cors = require("cors");
const app = express();
const port = 3000;

app.use(cors());

app.use(express.static("public"));
app.use(express.json());

// Enable CORS for all routes
app.options("*", cors());

app.get("/game", async (req, res) => {
  try {
    const response = await fetch("http://127.0.0.1:8080/game");
    const gameState = await response.json();
    res.json(gameState);
  } catch (error) {
    console.error("Express: Error fetching game state:", error);
    res.status(500).json({ error: "Could not fetch game state" });
  }
});

app.post("/register/:playerName", async (req, res) => {
  const { playerName } = req.params;
  try {
    const response = await fetch(`http://127.0.0.1:8080/register/${playerName}`, {
      method: "POST"
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

app.post("/remove-card", async (req, res) => {
  const { player, card } = req.body;
  console.log(`Express: Removing card ${card} for player ${player}`);
  try {
    const response = await fetch(`http://127.0.0.1:8080/remove-card/${player}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(card), // Match Rust's expected format
    });
    const updatedGameState = await response.json();
    res.json(updatedGameState);
  } catch (error) {
    console.error("Express: Error removing card:", error);
    res.status(500).json({ error: "Failed to remove card" });
  }
});

app.post("/reset-hand", async (req, res) => {
  const { player } = req.body;
  if (!player) {
    return res.status(400).json({ error: "Missing player name" });
  }
  
  console.log(`Express: Received reset-hand for player: ${player}`);
  try {
    const response = await fetch(`http://127.0.0.1:8080/reset-hand/${player}`, { 
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}" // Send empty JSON body to comply with Rust's expectations
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Rust server error: ${error}`);
    }

    const updatedGameState = await response.json();
    res.json(updatedGameState);
  } catch (error) {
    console.error("Express: Error resetting hand:", error);
    res.status(500).json({ error: error.message });
  }
});

app.listen(port, () => {
  console.log(`Express server running at http://127.0.0.1:${port}`);
});