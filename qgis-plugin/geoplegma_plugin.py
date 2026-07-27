import json
import os
from qgis.PyQt.QtWidgets import QAction, QFileDialog, QInputDialog, QMessageBox
from qgis.PyQt.QtGui import QIcon
from qgis.core import (
    QgsVectorLayer,
    QgsFeature,
    QgsGeometry,
    QgsProject,
    QgsField,
    QgsPointXY
)
from qgis.PyQt.QtCore import QVariant

try:
    from . import geoplegma_py
    import_error = None
except ImportError as e:
    geoplegma_py = None
    import_error = str(e)


class GeoPlegmaPlugin:
    def __init__(self, iface):
        self.iface = iface
        self.plugin_dir = os.path.dirname(__file__)
        self.action = None

    def initGui(self):
        self.action = QAction("Open GeoPlegma Store", self.iface.mainWindow())
        self.action.triggered.connect(self.run)
        
        self.iface.addToolBarIcon(self.action)
        self.iface.addPluginToMenu("&GeoPlegma", self.action)

    def unload(self):
        self.iface.removePluginMenu("&GeoPlegma", self.action)
        self.iface.removeToolBarIcon(self.action)

    def run(self):
        if geoplegma_py is None:
            QMessageBox.critical(
                self.iface.mainWindow(),
                "GeoPlegma Error",
                f"The geoplegma_py module could not be imported.\n\nDetails:\n{import_error}\n\nPlease ensure the python bindings are installed."
            )
            return

        store_path = QFileDialog.getExistingDirectory(
            self.iface.mainWindow(),
            "Select GeoPlegma Store Directory"
        )

        if not store_path:
            return

        try:
            store = geoplegma_py.Store(store_path)
            levels = store.levels()
            
            if not levels:
                QMessageBox.warning(self.iface.mainWindow(), "Warning", "No levels found in store.")
                return

            level_strs = [str(lvl) for lvl in levels]
            selected_level_str, ok = QInputDialog.getItem(
                self.iface.mainWindow(),
                "Select Resolution Level",
                "Available levels:",
                level_strs,
                0,
                False
            )

            if not ok or not selected_level_str:
                return
                
            level = int(selected_level_str)
            
            self.iface.mainWindow().statusBar().showMessage("Loading GeoPlegma data...")
            json_data = store.export_level(level)
            cells = json.loads(json_data)
            
            if not cells:
                QMessageBox.information(self.iface.mainWindow(), "Info", "No data found for this level.")
                return

            self._create_layer_from_cells(cells, store_path, level)
            self.iface.mainWindow().statusBar().showMessage("GeoPlegma data loaded successfully.", 5000)

        except Exception as e:
            QMessageBox.critical(
                self.iface.mainWindow(),
                "Error Loading Store",
                f"An error occurred: {str(e)}"
            )

    def _create_layer_from_cells(self, cells, store_path, level):
        layer_name = f"GeoPlegma_{os.path.basename(store_path)}_L{level}"

        layer = QgsVectorLayer("Polygon?crs=EPSG:4326", layer_name, "memory")
        provider = layer.dataProvider()

        if not cells:
            return

        first_cell = cells[0]
        bands = [k for k in first_cell.keys() if k != "polygon"]
        
        fields = [QgsField(band, QVariant.Double) for band in bands]
        provider.addAttributes(fields)
        layer.updateFields()

        features = []
        for cell in cells:
            feat = QgsFeature(layer.fields())
            
            polygon_coords = cell.get("polygon", [])
            points = [QgsPointXY(pt[0], pt[1]) for pt in polygon_coords]
            if points:
                if points[0] != points[-1]:
                    points.append(points[0])
                geom = QgsGeometry.fromPolygonXY([points])
                feat.setGeometry(geom)

            for band in bands:
                feat.setAttribute(band, cell.get(band))

            features.append(feat)

        provider.addFeatures(features)
        layer.updateExtents()
        
        from qgis.core import QgsFillSymbol, QgsProperty, QgsSymbolLayer, QgsSingleSymbolRenderer
        
        symbol = QgsFillSymbol.createSimple({'outline_style': 'no'})

        
        if "band_0" in bands and "band_1" in bands and "band_2" in bands:
            b0_val = first_cell.get("band_0", 0)
            multiplier = 255 if b0_val <= 1.0 else 1
            
            expr = f"""
            color_rgb(
                coalesce("band_0", 0) * {multiplier},
                coalesce("band_1", 0) * {multiplier},
                coalesce("band_2", 0) * {multiplier}
            )
            """
            prop = QgsProperty.fromExpression(expr)
            symbol.symbolLayer(0).setDataDefinedProperty(QgsSymbolLayer.PropertyFillColor, prop)
            
        layer.setRenderer(QgsSingleSymbolRenderer(symbol))
        layer.triggerRepaint()

        # Add layer to project
        QgsProject.instance().addMapLayer(layer)
