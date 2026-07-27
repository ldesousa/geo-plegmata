def classFactory(iface):
    from .geoplegma_plugin import GeoPlegmaPlugin
    return GeoPlegmaPlugin(iface)
