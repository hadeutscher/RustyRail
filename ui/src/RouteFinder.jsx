import React, { useState } from 'react';
import axios from 'axios';
import {
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  TextField,
  Button,
  Typography,
  List,
  ListItem,
  ListItemText,
} from '@mui/material';

const convertToIsoTime = (hhmmTime) => {
  const currentDate = new Date(); // Get the current date
  const timeParts = hhmmTime.split(':');

  return new Date(
    currentDate.getFullYear(),
    currentDate.getMonth(),
    currentDate.getDate(),
    parseInt(timeParts[0]),
    parseInt(timeParts[1])
  ).toISOString();
};

const convertToHHMM = (isoTime) => {
  const date = new Date(isoTime);
  const hours = date.getUTCHours().toString().padStart(2, '0');
  const minutes = date.getUTCMinutes().toString().padStart(2, '0');
  return `${hours}:${minutes}`;
};

const RouteFinder = ({ stations }) => {
  const [source, setSource] = useState('');
  const [destination, setDestination] = useState('');
  const [startTime, setStartTime] = useState('');
  const [endTime, setEndTime] = useState('');
  const [routes, setRoutes] = useState([]);

  const handleSearch = async () => {
    try {
      // Make API request to /harail/routes/find with selected parameters
      const response = await axios.get('/harail/routes/find', {
        params: {
          search: "Multi",
          start_station: source,
          start_time: convertToIsoTime(startTime),
          end_station: destination,
          end_time: convertToIsoTime(endTime),
        },
      });

      // Assuming the response contains an array of routes
      setRoutes(response.data);
    } catch (error) {
      console.error('Error fetching routes:', error);
    }
  };

  return (
    <div>
      <h2>Route Finder</h2>
      <FormControl>
        <InputLabel>Source station</InputLabel>
        <Select value={source} onChange={(e) => setSource(e.target.value)}>
          {stations.map((station) => (
            <MenuItem key={station.id} value={station.id}>
              {station.name}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl>
        <InputLabel>Destination station</InputLabel>
        <Select
          value={destination}
          onChange={(e) => setDestination(e.target.value)}
        >
          {stations.map((station) => (
            <MenuItem key={station.id} value={station.id}>
              {station.name}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <TextField
        label="Start time"
        variant="outlined"
        type="time"
        value={startTime}
        onChange={(e) => setStartTime(e.target.value)}
        fullWidth
      />
      <TextField
        label="End time"
        variant="outlined"
        type="time"
        value={endTime}
        onChange={(e) => setEndTime(e.target.value)}
        fullWidth
      />
      <Button variant="contained" onClick={handleSearch}>
        Search Routes
      </Button>

      {routes.length > 0 ? (
        <div>
          <Typography variant="h6">Routes:</Typography>
          <List>
            {routes.map((route) => (
              <ListItem>
                <List>
                  {route.parts.map((part) => (
                    <ListItem key={part.train}>
                      <ListItemText primary={
                        `${stations.find((station) => station.id === part.start_station).name} ל` +
                        `${stations.find((station) => station.id === part.end_station).name} ` +
                        `(${convertToHHMM(part.start_time)} - ${convertToHHMM(part.end_time)})`
                      } />
                    </ListItem>
                  ))}
                </List>
              </ListItem>
            ))}
          </List>
        </div>
      ) : (
        <Typography>No routes found.</Typography>
      )}
    </div>
  );
};

export default RouteFinder;
