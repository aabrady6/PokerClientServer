const express = require("express");
const cors = require("cors");
const app = express();
const port = 3000;

app.use(cors());

app.use(express.static("public"));
app.use(express.json());

// Enable CORS for all routes
app.options("*", cors());

app.post("/register/:playerName", async (req, res) => {
  const { playerName } = req.params;
  try {
    const response = await fetch(`http://localhost:8080/register/${playerName}`, {
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

app.get("/stats", async (req, res) => {
  console.log(req.query);
  const {
    type,
    stats_menu_type,
    selected_option,
  } = req.query;
  try {
    console.log("Getting stats with payload: ", type, stats_menu_type, selected_option);
    const response = await fetch(`http://localhost:8080/stats?type=${type}&stats_menu_type=${stats_menu_type}&selected_option=${selected_option}`, {
      method: "GET",
    });
    
    if (!response.ok) {
      const error = await response.text();
      return res.status(response.status).json({ error });
    }

    console.log("Stats response is ok!");
    
    const data = await response.json();
    res.json(data);
  } catch (error) {
    console.error("Get stats error:", error);
    res.status(500).json({ error: "Get stats failed" });
  }
});

app.listen(port, () => {
  console.log(`Express server running at http://127.0.0.1:${port}`);
});