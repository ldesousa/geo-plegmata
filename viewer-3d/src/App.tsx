// Copyright 2026 contributors to the GeoPlegmata project.
// Originally authored by Francisco Salgueiro, Instituto Superior Técnico (francisco.salgueiro@tecnico.ulisboa.pt)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

import { useState, useMemo, useEffect, useRef } from 'react';
import DeckGL from '@deck.gl/react';
import { _GlobeView as GlobeView, WebMercatorViewport } from '@deck.gl/core';
import { SolidPolygonLayer, GeoJsonLayer } from '@deck.gl/layers';

import { invoke } from '@tauri-apps/api/core';
import './App.css';

const BAND_KEY_PREFIX = 'band_';

function asUint8Array(payload: unknown): Uint8Array {
  if (payload instanceof Uint8Array) {
    return payload;
  }
  if (payload instanceof ArrayBuffer) {
    return new Uint8Array(payload);
  }
  if (Array.isArray(payload)) {
    return Uint8Array.from(payload);
  }
  throw new Error('Unexpected binary payload from backend');
}

function decodeBinary(payload: unknown) {
  const bytes = asUint8Array(payload);
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let offset = 0;
  const bandCount = view.getUint32(offset, true);
  offset += 4;
  const cellCount = view.getUint32(offset, true);
  offset += 4;

  const cells = new Array(cellCount);
  for (let index = 0; index < cellCount; index += 1) {
    const vertexCount = view.getUint16(offset, true);
    offset += 2;
    const polygon = new Array(vertexCount);
    for (let v = 0; v < vertexCount; v++) {
      const lon = view.getFloat64(offset, true);
      offset += 8;
      const lat = view.getFloat64(offset, true);
      offset += 8;
      polygon[v] = [lon, lat];
    }
    const cell: Record<string, any> = { polygon };

    for (let band = 0; band < bandCount; band += 1) {
      const value = view.getFloat64(offset, true);
      offset += 8;
      if (Number.isFinite(value)) {
        cell[`${BAND_KEY_PREFIX}${band}`] = value;
      }
    }

    cells[index] = cell;
  }

  return { bandCount, cells };
}

const COUNTRIES_BORDERS_URL = '/countries.geojson';




const earthMaskLayer = new GeoJsonLayer({
  id: 'EarthMaskLayer',
  data: {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        properties: {},
        geometry: {
          type: 'Polygon',
          coordinates: [[
            [-180, -90],
            [-180, 90],
            [180, 90],
            [180, -90],
            [-180, -90]
          ]]
        }
      }
    ]
  },
  stroked: false,
  filled: true,
  getFillColor: [255, 255, 255, 255],
  pickable: false
});

const countriesBordersLayer = new GeoJsonLayer({
  id: 'CountriesBordersLayer',
  data: COUNTRIES_BORDERS_URL,
  stroked: true,
  filled: false,
  lineWidthMinPixels: 1,
  getLineColor: [120, 132, 145, 120],
  pickable: false
});

function App() {
  const [storePath, setStorePath] = useState('./tmp/gp_encoding_geotiff_convert');
  const [levels, setLevels] = useState<number[]>([]);
  const [level, setLevel] = useState<number | null>(null);
  const [cells, setCells] = useState<any[]>([]);
  const timeoutRef = useRef<number | null>(null);
  const [bandCount, setBandCount] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [viewState, setViewState] = useState({
    longitude: 0,
    latitude: 0,
    zoom: 1,
    maxZoom: 20,
    pitch: 30,
    bearing: 0
  });
  const [error, setError] = useState<string | null>(null);

  const resolveDefaultLevel = (availableLevels: number[]) => {
    if (availableLevels.length === 0) {
      return null;
    }
    const sortedLevels = [...availableLevels].sort((a, b) => a - b);
    return sortedLevels[sortedLevels.length - 1];
  };

  const fetchLevels = async () => {
    try {
      const storeLevels: number[] = await invoke('get_levels', { store: storePath });
      const sortedLevels = [...storeLevels].sort((a, b) => a - b);
      setLevels(sortedLevels);
      setLevel((current) => {
        if (current !== null && sortedLevels.includes(current)) {
          return current;
        }
        return resolveDefaultLevel(sortedLevels);
      });
      return sortedLevels;
    } catch (err: any) {
      console.error(err);
      setError(err.toString());
      setLevels([]);
      setLevel(null);
      return [];
    }
  };

  useEffect(() => {
    fetchLevels();
  }, [storePath]);

  const resolveLevelForZoom = (zoom: number, availableLevels: number[]) => {
    if (availableLevels.length === 0) return null;
    const target = Math.max(0, Math.floor(zoom));
    let best = availableLevels[0];
    for (const lvl of availableLevels) {
      if (lvl <= target) {
        best = lvl;
      }
    }
    return best;
  };

  const loadDataForCurrentView = async () => {
    if (levels.length === 0) return;
    setError(null);
    setIsLoading(true);
    try {
      const resolvedLevel = resolveLevelForZoom(viewState.zoom, levels);
      if (resolvedLevel === null) {
        throw new Error('No levels found in the selected store');
      }

      const width = window.innerWidth;
      const height = window.innerHeight;
      const viewport = new WebMercatorViewport({ ...viewState, width, height });
      const bounds = viewport.getBounds();

      const payload = await invoke('get_data_binary', { 
        store: storePath, 
        level: resolvedLevel,
        bbox: [bounds[0], bounds[1], bounds[2], bounds[3]]
      });
      const { cells: decodedCells, bandCount: decodedBandCount } = decodeBinary(payload);
      setCells(decodedCells);
      setBandCount(decodedBandCount);
      setLevel(resolvedLevel);
    } catch (err: any) {
      console.error(err);
      setError(err.toString());
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (storePath && levels.length > 0) {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      timeoutRef.current = window.setTimeout(() => {
        loadDataForCurrentView();
      }, 500);
    }
  }, [viewState, levels, storePath]);

  const layers = useMemo(() => {
    const layerList: any[] = [countriesBordersLayer, earthMaskLayer];

    if (cells.length > 0) {
      const useElevation = bandCount === 1;

      const polygonLayer = new SolidPolygonLayer({
        id: 'SolidPolygonLayer',
        data: cells,
        elevationScale: 20,
        extruded: useElevation,
        filled: true,
        getElevation: useElevation ? (d: any) => d.band_0 / 2 : 0,
        getFillColor: useElevation
          ? (d: any) => [d.band_0, d.band_0, d.band_0]
          : (d: any) => [d.band_0, d.band_1, d.band_2],
        getPolygon: (d: any) => d.polygon,
        wireframe: false,
        pickable: true,
      });

      layerList.push(polygonLayer);
    }

    return layerList;
  }, [cells, bandCount]);

  return (
    <div className="app-root" onContextMenu={(e) => e.preventDefault()}>
      <DeckGL
        views={new GlobeView({ id: 'globe' })}
        initialViewState={viewState}
        onViewStateChange={({ viewState }: any) => setViewState(viewState as any)}
        controller={true}
        layers={layers}
        getTooltip={({ object }: any) => object && `${object.hex}`}
        style={{ backgroundColor: 'transparent' }}
      />

      <div className="app-shell">
        <div className="shell-middle">
          <aside className="control-panel">
            <div className="input-group">
              <label>Store Path</label>
              <input
                type="text"
                value={storePath}
                onChange={e => setStorePath(e.target.value)}
                placeholder="/path/to/zarr"
              />
            </div>
            <div className="input-group">
              <label>Active Level</label>
              <input
                type="text"
                value={level !== null ? level : ''}
                disabled
              />
            </div>
            <div className="panel-actions">
              <button onClick={loadDataForCurrentView} disabled={isLoading || levels.length === 0}>
                {isLoading ? 'Loading...' : 'Reload Data'}
              </button>
            </div>
            {error && <div className="error">{error}</div>}
          </aside>
        </div>
      </div>
    </div>
  );
}

export default App;
