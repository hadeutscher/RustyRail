import { useState, useEffect } from "react";
import { Container, Card } from "@mui/material";
import axios from "axios";
import RouteFinder from "./RouteFinder.jsx";

const App = () => {
  const [stations, setStations] = useState([]);

  useEffect(() => {
    const fetchStations = async () => {
      try {
        const response = await axios.get("/harail/stations");
        setStations(response.data);
      } catch (error) {
        console.error("Error fetching stations:", error);
      }
    };

    fetchStations();
  }, []);

  return (
    <div>
      <Container>
        <h1>HaRail</h1>
        <Card>
          <RouteFinder stations={stations} />
        </Card>
      </Container>
    </div>
  );
};

export default App;
