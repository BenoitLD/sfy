import {Component} from 'inferno';
import L from 'leaflet';

import {Buoy} from 'models';

import './BuoyIndex.scss';
import 'leaflet/dist/leaflet.css';
import 'leaflet/dist/images/marker-icon-2x.png';
import 'leaflet/dist/images/marker-shadow.png';
import 'leaflet/dist/images/marker-shadow.png';

const MAPBOX_TOKEN: string = 'pk.eyJ1IjoiZ2F1dGVoIiwiYSI6ImNreWZ2MWd4NjBxNnoyb3M4eWRqNjlmMGMifQ.m-5Q9BBf2yQxp1fGStxYRg';

interface Props {
  buoys: Buoy[];
}

interface State {
}

export class BuoyMap
  extends Component<Props, State>
{

  public state = {};
  map = {};

  constructor(props: Props, context) {
    super(props, context);
  }

  componentDidMount() {
    console.log("loading leaflet..");
    this.map = L.map('map').setView([60.11304848114283, 2.3882482939071434], 5);

    L.tileLayer('https://api.mapbox.com/styles/v1/{id}/tiles/{z}/{x}/{y}?access_token={access_token}', {
      maxZoom: 18,
      attribution: 'Map data &copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors, ' +
        'Imagery © <a href="https://www.mapbox.com/">Mapbox</a>',
      id: 'mapbox/satellite-streets-v11',
      tileSize: 512,
      zoomOffset: -1,
      access_token: MAPBOX_TOKEN
    }).addTo(this.map);
  }

  componentDidUpdate(props) {
    for (const buoy of props.buoys) {
      let marker = L.marker([buoy.any_lat(), buoy.any_lon()]).addTo(this.map);
      marker.bindTooltip(`${buoy.sn} (${buoy.dev})`);
    }
  }

  public render() {
    return (
      <div id="map" class="container-fluid">
      </div>
    );
  }
}

