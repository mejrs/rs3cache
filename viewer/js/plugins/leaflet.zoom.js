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
    L.Control.CustomZoom = L.Control.Zoom.extend({
        options: {
            position: 'topright',

            zoomIn: {
                innerHTML: '+',
                title: 'Zoom in',
                role: "button"
            },
            zoomOut: {
                innerHTML: '-',
                title: 'Zoom out',
                role: "button"
            },
            zoomReset: {
                innerHTML: 'test',
                title: 'Reset zoom',
                role: "button"
            },
            defaultZoom: 2,
            displayZoomLevel: true,
            className: 'leaflet-control-zoom',
        },

        initialize: function (options) {
            L.setOptions(this, options);
            this._map = null;
        },

        onAdd: function (map) {
            this._map = map;

            this._container = L.DomUtil.create('div', 'leaflet-control-zoom leaflet-bar');

            this._zoomInButton = this.createElement('a', this.options.zoomIn, this.options.className + '-in', this._container, {
                "click": this._zoomIn
            });
            if (this.options.displayZoomLevel) {
                this._zoomLevel = this.createElement('a', this.options.zoomReset, this.options.className + '-level', this._container, {
                    "click": this._resetZoom
                });
            }
            this._zoomOutButton = this.createElement('a', this.options.zoomOut, this.options.className + '-out', this._container, {
                "click": this._zoomOut
            });

            this._update();
            this._map.on('zoomend zoomlevelschange', this._update, this);

            return this._container;
        },

        //(<string> tagname, <object> attributes including innerHTML, <string> className, parent DOM element, {triggerevent:fn})
        createElement: function (tag, attributes, className, container, eventfnpairs) {
            let htmlElement = L.DomUtil.create(tag, className, container);

            if (attributes !== null) {
                for (const[property, value]of Object.entries(attributes)) {
                    htmlElement[property] = value;
                }
            }

            if (eventfnpairs !== undefined) {
                for (const[event, fn]of Object.entries(eventfnpairs)) {
                    L.DomEvent
                    .disableClickPropagation(htmlElement)
                    .on(htmlElement, event, fn, this)
                    .on(htmlElement, event, this._refocusOnMap, this)
                }
            }
            return htmlElement;
        },

        _update: function () {

            L.Control.Zoom.prototype._updateDisabled.call(this);
            if (this.options.displayZoomLevel) {

                // Update displayed zoom level
                this._zoomLevel.textContent = (this.getZoomPercentage() * 100) + '%';
            }
        },

        getZoomPercentage: function (zoom) {
            if (!zoom) {
                zoom = this.options.defaultZoom;
            }
            return 1 / this._map.getZoomScale(zoom);
        },

        _resetZoom: function () {
            this._map.setZoom(this.options.defaultZoom);
        },
    });

    L.Map.mergeOptions({
        zoomControl: false
    });

    L.Map.addInitHook(function () {
        if (this.options.customZoomControl) {
            this.customZoomControl = new L.Control.CustomZoom(this.options.customZoomControl);
            this.addControl(this.customZoomControl)
        }

    });

    L.control.customZoom = function (options) {
        return new L.Control.CustomZoom(options);
    }
});
