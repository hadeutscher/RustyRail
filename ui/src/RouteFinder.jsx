import { useState } from "react";
import axios from "axios";
import PropTypes from "prop-types";
import {
  Card,
  MenuItem,
  TextField,
  Button,
  Typography,
  List,
  ListItem,
  ListItemText,
} from "@mui/material";
import { DatePicker } from "@mui/x-date-pickers/DatePicker";
import dayjs from "dayjs";

const convertToIsoTime = (date, hhmmTime) => {
  const timeParts = hhmmTime.split(":");

  return new Date(
    Date.UTC(
      date.year(),
      date.month(),
      date.date(),
      parseInt(timeParts[0]),
      parseInt(timeParts[1])
    )
  ).toISOString();
};

const convertToHHMM = (isoTime) => {
  const date = new Date(isoTime);
  const hours = date.getUTCHours().toString().padStart(2, "0");
  const minutes = date.getUTCMinutes().toString().padStart(2, "0");
  return `${hours}:${minutes}`;
};

const RouteFinder = ({ stations }) => {
  const [source, setSource] = useState("");
  const [destination, setDestination] = useState("");
  const [date, setDate] = useState(dayjs());
  const [startTime, setStartTime] = useState("");
  const [endTime, setEndTime] = useState("");
  const [routes, setRoutes] = useState([]);

  const sortStations = (stations) => {
    return stations.sort((a, b) => {
      if (a.name < b.name) {
        return -1;
      }
      if (a.name > b.name) {
        return 1;
      }
      return 0;
    });
  };

  const handleSearch = async () => {
    try {
      // Make API request to /harail/routes/find with selected parameters
      const response = await axios.get("/harail/routes/find", {
        params: {
          search: "Multi",
          start_station: source,
          start_time: convertToIsoTime(date, startTime),
          end_station: destination,
          end_time: convertToIsoTime(date, endTime),
        },
      });

      // Assuming the response contains an array of routes
      setRoutes(response.data);
    } catch (error) {
      console.error("Error fetching routes:", error);
    }
  };

  return (
    <div>
      <h2>Route Finder</h2>
      <TextField
        select
        label="Source station"
        onChange={(e) => setSource(e.target.value)}
      >
        {sortStations(stations).map((station) => (
          <MenuItem key={station.id} value={station.id}>
            {station.name}
          </MenuItem>
        ))}
      </TextField>
      <TextField
        select
        label="Destination station"
        onChange={(e) => setDestination(e.target.value)}
      >
        {sortStations(stations).map((station) => (
          <MenuItem key={station.id} value={station.id}>
            {station.name}
          </MenuItem>
        ))}
      </TextField>
      <DatePicker
        label="Date"
        value={date}
        onChange={(date) => setDate(date)}
      />
      <TextField
        label="Start time"
        variant="outlined"
        type="time"
        value={startTime}
        slotProps={{ inputLabel: { shrink: true } }}
        onChange={(e) => setStartTime(e.target.value)}
      />
      <TextField
        label="End time"
        variant="outlined"
        type="time"
        value={endTime}
        slotProps={{ inputLabel: { shrink: true } }}
        onChange={(e) => setEndTime(e.target.value)}
      />
      <Button variant="contained" onClick={handleSearch}>
        Search Routes
      </Button>

      {routes.length > 0 ? (
        <div>
          <Typography variant="h6">Routes:</Typography>
          {routes.map((route) => (
            <Card key={route.parts[0].train} variant="outlined">
              <List>
                {route.parts.map((part) => (
                  <ListItem key={part.train}>
                    <ListItemText
                      primary={
                        `${stations.find((station) => station.id === part.start_station).name} ל` +
                        `${stations.find((station) => station.id === part.end_station).name} ` +
                        `(${convertToHHMM(part.start_time)} - ${convertToHHMM(part.end_time)})`
                      }
                    />
                  </ListItem>
                ))}
              </List>
            </Card>
          ))}
        </div>
      ) : (
        <Typography>No routes found.</Typography>
      )}
    </div>
  );
};

RouteFinder.propTypes = {
  stations: PropTypes.array,
};

export default RouteFinder;
