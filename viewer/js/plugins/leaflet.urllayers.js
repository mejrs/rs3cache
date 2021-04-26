(function (factory) {
    var L;
    if (typeof define === "function" && define.amd) {
        define(["leaflet"], factory)
    } else if (typeof module !== "undefined") {
        L = require("leaflet");
        module.exports = factory(L)
    } else {
        if (typeof window.L === "undefined") {
            throw new Error("Leaflet must be loaded first")
        }
        factory(window.L)
    }
})(function (L) {
    L.Control.Layers.UrlParam = L.Control.Layers.extend({
        onAdd: function (map) {
            this.initParamLayers(map);
            return L.Control.Layers.prototype.onAdd.call(this, map);
        },

        initParamLayers: function (map) {
            let url = new URL(window.location.href);
            let params = url.searchParams;
            let initLayers = params.getAll('layer');

            for (const overlay of this._layers.filter(layer => layer.overlay)) {
                if (initLayers.includes(overlay.name)) {
                    overlay.layer.addTo(map);
                }
            }
        },

        addSearchParam(layerName) {
            let url = new URL(window.location.href);
            let params = url.searchParams;
            params.append('layer', layerName)
            url.search = params;
            history.replaceState(0, "Location", url);
        },

        removeSearchParam(layerName) {
            let url = new URL(window.location.href);
            let params = url.searchParams;
            let otherLayers = params.getAll('layer').filter(layer => layer !== layerName);

            params.delete('layer');
            for (const layer of otherLayers) {
                params.append('layer', layer)
            }
            url.search = params;
            history.replaceState(0, "Location", url);
        },

        _onLayerChange: function (e) {
            let layerName = this._getLayer(L.Util.stamp(e.target)).name;
            if (e.type === 'add') {
                this.addSearchParam(layerName);
            } else if (e.type === 'remove') {
                this.removeSearchParam(layerName);
            }
            return L.Control.Layers.prototype._onLayerChange.call(this, e);
        }
    });

    L.control.layers.urlParam = function (baseLayers, overlays, options) {
        return new L.Control.Layers.UrlParam(baseLayers, overlays, options);
    };
});
