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
    L.Control.Plane = L.Control.extend({
        options: {
            position: 'topright',
            upicon: '<svg viewBox="0 0 64 64" height="24px" width="24px"><g style="display:inline" transform="translate(0,-233)"> <path d="m 27,238 -19,-0 7,7 -11,11 5,5 11,-11 7,7 z" style="fill:#000000;fill-opacity:1" /><path d="M 4,61 V 47 H 19 V 33 H 33 V 18 H 47 V 4 H 61 V 12 L 12,61 Z" style="display:inline;fill:#000000" transform="translate(0,233)"/></g></svg>',
            downicon: '<svg viewBox="0 0 64 64" height="24px" width="24px"><g style="display:inline" transform="translate(0,-233)"> <path d="m 4,261 19,0 -7,-7 11,-11 -5,-5 -11,11 -7,-7 z" style="fill:#000000;fill-opacity:1" /><path d="M 4,61 V 47 H 19 V 33 H 33 V 18 H 47 V 4 H 61 V 12 L 12,61 Z" style="display:inline;fill:#000000" transform="translate(0,233)"/></g></svg>',
        },

        initialize: function (options) {
            L.setOptions(this, options);
            this._map = null;
        },

        createElement: function (tag, html, title, className, container, fn, event) {
            let htmlelement = L.DomUtil.create(tag, className, container);
            htmlelement.innerHTML = html;
            htmlelement.title = title;

            L.DomEvent
            .disableClickPropagation(htmlelement)
            .on(htmlelement, event, fn, this)
            .on(htmlelement, event, this._refocusOnMap, this)

            return htmlelement;
        },

        onAdd: function (map) {
            this._map = map;

            let containerName = 'leaflet-control-plane';
            let container = L.DomUtil.create('div', containerName + ' leaflet-bar');
            let className = 'leaflet-disabled';

            let listenerUp = () => this._map.setPlane(this._map._plane + 1);
            let listenerDown = () => this._map.setPlane(this._map._plane - 1);
            let listenerLabel = () => this._map.setPlane(this.options.planeMin || 0); // Reset plane

            this._buttonUp = this.createElement('a', this.options.upicon, 'Move up', containerName + '-up ' + (this._map._plane + 1 > this.options.planeMax ? className : ''), container, listenerUp, 'click');
            this._buttonPlane = this.createElement('a', this._map._plane, 'Current plane', containerName + '-plane', container, listenerLabel, 'click');
            this._buttonDown = this.createElement('a', this.options.downicon, 'Move down', containerName + '-down ' + (this._map._plane - 1 < this.options.planeMin ? className : ''), container, listenerDown, 'click');

            this._map.on('planechange maxplanechange', this.updateButtons, this);
            L.DomUtil.disableTextSelection();

            return container;
        },

        updateButtons: function () {
            let plane = this._map._plane;
            let maxPlane = this._map.getMaxPlane();
            let minPlane = this._map.getMaxPlane();

            this._buttonPlane.textContent = plane;

            // Disable buttons
            L.DomUtil.removeClass(this._buttonUp, 'leaflet-disabled');
            L.DomUtil.removeClass(this._buttonDown, 'leaflet-disabled');
            L.DomUtil.removeClass(this._buttonPlane, 'leaflet-disabled');

            if (plane === minPlane) {
                L.DomUtil.addClass(this._buttonPlane, 'leaflet-disabled');
            }

            if (plane - 1 < minPlane) {
                L.DomUtil.addClass(this._buttonDown, 'leaflet-disabled');
            }
            if (plane + 1 > maxPlane) {
                L.DomUtil.addClass(this._buttonUp, 'leaflet-disabled');
            }
        },
    });

    L.Map.mergeOptions({
        zoomControl: false
    });

    L.Map.addInitHook(function () {
        if (this.options.planeControl) {
            this.planeControl = new L.Control.Plane(this.options.planeControl);
            this.addControl(this.planeControl)
        }
    });

    L.control.plane = function (options) {
        return new L.Control.Plane(options);
    };
});
