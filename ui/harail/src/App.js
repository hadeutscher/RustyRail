// src/App.js
import React, { useState, useEffect } from 'react';
import axios from 'axios';
import RouteFinder from './RouteFinder';

const App = () => {
  const [stations, setStations] = useState([]);

  useEffect(() => {
    const fetchStations = async () => {
      try {
        const response = await axios.get('/harail/stations');
        setStations(response.data);
      } catch (error) {
        console.error('Error fetching stations:', error);
      }
    };

    fetchStations();
  }, []);

  return (
    <div className="App">
      <h1>HaRail</h1>
      <RouteFinder stations={stations} />
    </div>
  );
};

export default App;
