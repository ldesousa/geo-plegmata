import test from 'ava'

import { Dggrs, defaultConfig } from '..'
import type { FlatZones } from '..'

const DGGRS = 'IVEA7H'
const RL = 3
const BBOX = [[-10.0, -10.0], [10.0, 10.0]]
const POINT = [9.06, 52.98]

function decodeId(flat: FlatZones, i: number): string {
  const start = flat.idOffsets[i]
  const end = i + 1 < flat.idOffsets.length ? flat.idOffsets[i + 1] : flat.utf8Ids.length
  return Buffer.from(flat.utf8Ids.slice(start, end)).toString('utf8')
}

test('zonesFromBbox - parallel arrays have matching length', (t) => {
  const g = new Dggrs(DGGRS)
  const flat = g.zonesFromBbox(RL, BBOX)
  const n = flat.idOffsets.length

  t.true(n > 0, 'should return at least one zone')
  t.is(flat.centerX.length, n)
  t.is(flat.centerY.length, n)
  t.is(flat.vertexCount.length, n)
  t.is(flat.regionOffsets.length, n)
  t.is(flat.childrenOffsets.length, n)
  t.is(flat.neighborsOffsets.length, n)
  t.is(flat.areaSqm.length, n)
})

test('zonesFromBbox - region coords are x,y pairs with valid lat/lon', (t) => {
  const g = new Dggrs(DGGRS)
  const flat = g.zonesFromBbox(RL, BBOX)

  t.is(flat.regionCoords.length % 2, 0, 'regionCoords must be x,y pairs')

  for (let i = 0; i < flat.regionCoords.length; i += 2) {
    const lon = flat.regionCoords[i]
    const lat = flat.regionCoords[i + 1]
    t.true(lon >= -180 && lon <= 180, `lon ${lon} out of range`)
    t.true(lat >= -90 && lat <= 90, `lat ${lat} out of range`)
  }
})

test('zonesFromBbox - zone IDs decode to non-empty strings', (t) => {
  const g = new Dggrs(DGGRS)
  const flat = g.zonesFromBbox(RL, BBOX)

  for (let i = 0; i < flat.idOffsets.length; i++) {
    const id = decodeId(flat, i)
    t.true(id.length > 0, `zone ${i} has empty ID`)
  }
})

test('zoneFromPoint - returns exactly one zone', (t) => {
  const g = new Dggrs(DGGRS)
  const flat = g.zoneFromPoint(RL, POINT)

  t.is(flat.idOffsets.length, 1)
  t.is(flat.centerX.length, 1)
  t.is(flat.centerY.length, 1)
})

test('zoneFromId - roundtrip returns same zone', (t) => {
  const g = new Dggrs(DGGRS)
  const byPoint = g.zoneFromPoint(RL, POINT)
  const id = decodeId(byPoint, 0)

  const byId = g.zoneFromId(id)

  t.is(byId.idOffsets.length, 1)
  t.is(decodeId(byId, 0), id)
  t.is(byId.centerX[0], byPoint.centerX[0])
  t.is(byId.centerY[0], byPoint.centerY[0])
})

test('zonesFromParent - returns child zones', (t) => {
  const g = new Dggrs(DGGRS)
  const parent = g.zoneFromPoint(RL, POINT)
  const parentId = decodeId(parent, 0)

  const children = g.zonesFromParent(1, parentId)

  t.true(children.idOffsets.length > 0, 'should return at least one child zone')
})

test('zoneCount - returns positive integer at given refinement level', (t) => {
  const g = new Dggrs(DGGRS)
  const count = g.zoneCount(RL)

  t.true(count > 0)
  t.is(count, Math.round(count))
})

test('refinement level bounds are self-consistent', (t) => {
  const g = new Dggrs(DGGRS)
  const min = g.minRefinementLevel()
  const max = g.maxRefinementLevel()
  const def = g.defaulRefinementLevel()

  t.true(min <= def, `default (${def}) should be >= min (${min})`)
  t.true(def <= max, `default (${def}) should be <= max (${max})`)
})

test('defaultConfig - all fields are true', (t) => {
  const config = defaultConfig()

  t.true(config.region)
  t.true(config.center)
  t.true(config.vertexCount)
  t.true(config.children)
  t.true(config.neighbors)
  t.true(config.areaSqm)
  t.true(config.densify)
})

test('config center:false - center arrays are empty', (t) => {
  const g = new Dggrs(DGGRS)
  const flat = g.zonesFromBbox(RL, BBOX, { ...defaultConfig(), center: false })

  t.is(flat.centerX.length, 0)
  t.is(flat.centerY.length, 0)
})
