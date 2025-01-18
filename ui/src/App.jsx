import { useState, useEffect } from "react";
import { Container, Card } from "@mui/material";
import axios from "axios";
import RouteFinder from "./RouteFinder.jsx";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider";
import { AdapterDayjs } from "@mui/x-date-pickers/AdapterDayjs";
import "dayjs/locale/en-il";

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
      <LocalizationProvider dateAdapter={AdapterDayjs} adapterLocale="en-il">
        <Container>
          <h1>HaRail</h1>
          <Card>
            <RouteFinder stations={stations} />
          </Card>
        </Container>
      </LocalizationProvider>
    </div>
  );
};

export default App;
