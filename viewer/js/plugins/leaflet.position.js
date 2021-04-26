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
    L.Control.Position = L.Control.extend({
        options: {
            position: 'topleft',
            separator: '_',
            emptyString: 'Unavailable',
            prefix: "",
            flyDuration: 3, //seconds
        },

        initialize: function (options) {
            L.setOptions(this, options);
            this._map = null;

        },

        onAdd: function (map) {
            this._map = map;
            this._container = L.DomUtil.create('div', 'leaflet-control-mouseposition');
            this._rect = L.rectangle([[0, 0], [1, 1]], {
                color: "#ff7800",
                weight: 1,
            });

            // gross hack because the Path renderer doesn't work properly if this is added right now
            // we delay this for after this event loop finishes
            setTimeout(() => {
                this._rect.addTo(map);
            }, 0)

            L.DomEvent.disableClickPropagation(this._container);
            L.DomEvent.on(this._container, 'click', this._onSelect, this);
            L.DomUtil.disableTextSelection();

            this._map.on('mousemove', this._updateContainerPointCache, this);
            this._map.on('move moveend zoom zoomend resize', this.redrawRect, this);
            this._map.on('mouseout', this.clear.bind(this));

            this._container.innerHTML = this.options.emptyString;

            return this._container;
        },

        clear: function () {
            this._container.innerHTML = this.options.emptyString;
            // hack to make it 'disappear'...not calling .remove()
            // because otherwise `redrawRect` would have to
            // take care of readding it to the map
            this._rect.setBounds([[-1000, -1000], [-1000, -1000]]);
        },

        onRemove: function (map) {
            map.off('mousemove', this._update);
            this.rect.remove();

        },

        convert: function (_plane, _globalX, _globalY) {
            return {
                plane: _plane,
                i: _globalX >> 6,
                j: _globalY >> 6,
                x: _globalX & 0x3F,
                y: _globalY & 0x3F,
            }
        },

        deConvert: function (_plane, _i, _j, _x, _y) {
            return {
                plane: _plane,
                globalX: _i << 6 | _x,
                globalY: _j << 6 | _y
            }
        },

        interpret: function (input) {
            let numbers = input.match(/\d+/g).map(Number);
            if (numbers.length == 1) {
                return {
                    plane: numbers[0] >> 28,
                    globalX: (numbers[0] >> 14) & 0x3FFF,
                    globalY: numbers[0] & 0x3FFF
                }
            } else if (numbers.length >= 2) {
                numbers.push(0, 0, 0);
                if (!(numbers[0]in[0, 1, 2, 3])) {
                    numbers.unshift(this._map.getPlane());
                }
                if (numbers[1] > 200 || numbers[2] > 200) {
                    return {
                        plane: numbers[0],
                        globalX: numbers[1],
                        globalY: numbers[2]
                    }
                } else {
                    return {
                        plane: numbers[0],
                        globalX: numbers[1] << 6 | numbers[3],
                        globalY: numbers[2] << 6 | numbers[4]
                    }
                }
            }
            return undefined;
        },

        _onSelect: function (e) {
            let input = prompt("Go to:");
            if (input) {
                let destination = this.interpret(input);
                if (this.validateCoordinate(destination)) {
                    this.panMap(destination);
                    this.placeCrosshair(destination);
                } else {
                    console.error(input, "was parsed as", this.createString(destination), "which is not a valid coordinate.");
                }
            }
        },

        panMap: function (destination) {
            this._map.setPlane(destination.plane);
            this._map.flyTo([destination.globalY, destination.globalX], 3, {
                duration: this.options.flyDuration,
                animate: false,
            });
            this._map.once('moveend', function () {
                this.fire("panend");
            });
        },

        placeCrosshair: function (destination) {
            let icon = L.icon({
                iconUrl: 'sprites/22449-0.png',
                iconAnchor: [25, 25]
            });
            let marker = L.marker(L.latLng(destination.globalY + 0.5, destination.globalX + 0.5), {
                icon: icon
            });
            marker.addTo(this._map);
            setTimeout(() => {
                if (marker) {
                    marker.remove();
                }
            }, 50000);
        },

        validateCoordinate: function (destination) {
            return destination && destination.plane < 4 && this._map.options.maxBounds.contains(L.latLng(destination.globalY, destination.globalX));
        },

        createString: function (...args) {
            if (typeof args[0] === "number") {
                return args.join(this.options.separator);
            }
            if (typeof args[0] === "object") {
                let coord = args[0];
                return [coord.plane, coord.i, coord.j, coord.x, coord.y, coord.globalX, coord.globalY].filter(item => item !== undefined).join(this.options.separator);
            }
        },
        _containerPointCache: {
            x: 0,
            y: 0
        },

        _updateContainerPointCache: function (e) {
            this._containerPointCache = e.containerPoint;
            this.redrawRect();

        },
        redrawRect: function () {
            let position = this._map.containerPointToLatLng(this._containerPointCache);
            this.globalX = parseInt(position.lng);
            this.globalY = parseInt(position.lat);
            let jCoord = this.createString(this.convert(this._map._plane, this.globalX, this.globalY));
            let pxyCoord = this.createString(this._map._plane, this.globalX, this.globalY);
            this._container.innerHTML = jCoord + "<br>" + pxyCoord;
            this._rect.setBounds([[this.globalY, this.globalX], [this.globalY + 1, this.globalX + 1]])
        }
    });

    L.Map.addInitHook(function () {
        if (this.options.positionControl) {
            this.positionControl = new L.Control.Position(this.options.positionControl);
            this.addControl(this.positionControl)
        }
    });

    L.control.position = function (options) {
        return new L.Control.Position(options);
    };
});
