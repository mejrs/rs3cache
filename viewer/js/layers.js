'use strict';

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
    L.GameMap = L.Map.extend({
        initialize: function (id, options) { // (HTMLElement or String, Object)

            let parsedUrl = new URL(window.location.href);

            options.zoom = Number(parsedUrl.searchParams.get('zoom') || parsedUrl.searchParams.get('z') || this._limitZoom(options.zoom) || 0);

            this._plane = Number(parsedUrl.searchParams.get('plane') || parsedUrl.searchParams.get('p') || this._limitPlane(options.plane) || 0);

            this._mapId = Number(parsedUrl.searchParams.get('mapId') || parsedUrl.searchParams.get('mapid') || parsedUrl.searchParams.get('m') || options.initialMapId || -1);
            options.x = Number(parsedUrl.searchParams.get('x')) || options.x || 3232;
            options.y = Number(parsedUrl.searchParams.get('y')) || options.y || 3232;
            options.center = [options.y, options.x];

            options.crs = L.CRS.Simple;

            L.Map.prototype.initialize.call(this, id, options);

            this.on('moveend planechange mapidchange', this.setSearchParams)

            if (this.options.baseMaps) {
                fetch(this.options.baseMaps).then(response => response.json()).then(data => {

                    this._baseMaps = Array.isArray(data) ? this.castBaseMaps(data) : data;
                    this._allowedMapIds = Object.keys(this._baseMaps).map(Number);
                    let bounds = this.getMapIdBounds(this._mapId);

                    if (options.showMapBorder) {
                        this.boundsRect = L.rectangle(bounds, {
                            color: "#ffffff",
                            weight: 1,
                            fill: false,
                            smoothFactor: 1,
                        }).addTo(this);
                    }

                    let paddedBounds = bounds.pad(0.1);
                    this.setMaxBounds(paddedBounds);
                });
            }

            if (options.messageBox) {
                this._messageContainer = L.DomUtil.create('div', 'leaflet-control-message-container');
                this._controlContainer.appendChild(this._messageContainer);
            }

        },

        addMessage: function (message, duration = 4000) {
            if (this.options.messageBox) {
                let messageBox = L.DomUtil.create('div', 'leaflet-control-message-box');

                let messageContent = L.DomUtil.create('div', 'leaflet-control-message-content');
                messageContent.innerHTML = message;
                messageBox.appendChild(messageContent);

                let clearButton = L.DomUtil.create('div', 'leaflet-control-message-clear');
                clearButton.innerHTML = "[dismiss]";
                clearButton.onclick = () => this._messageContainer.removeChild(messageBox);
                messageBox.appendChild(clearButton);

                this._messageContainer.appendChild(messageBox);
                setTimeout(() => {
                    if (this._messageContainer.contains(messageBox)) {
                        this._messageContainer.removeChild(messageBox);
                    }
                }, duration);
                return messageBox;
            } else {
                console.log(message);
            }
        },

        castBaseMaps: function (data) {
            let baseMaps = {}
            for (let i in data) {
                baseMaps[data[i].mapId] = data[i];
            }
            return baseMaps;

        },

        setSearchParams: function (e, parameters = {
                m: this._mapId,
                z: this._zoom,
                p: this._plane,
                x: Math.round(this.getCenter().lng),
                y: Math.round(this.getCenter().lat)
            }) {
            let url = new URL(window.location.href);
            let params = url.searchParams;

            for (const param in["mapId", "mapid", "zoom", "plane"]) {
                params.delete(param)
            }

            for (let[key, value]of Object.entries(parameters)) {
                params.set(key, value);
            }
            url.search = params;
            history.replaceState(0, "Location", url);
        },

        _limitPlane: function (plane) {
            //todo process allowedPlanes in basemap data
            var min = this.getMinPlane();
            var max = this.getMaxPlane();
            return Math.max(min, Math.min(max, plane));
        },

        _validateMapId: function (_mapId) {
            const parsedMapId = parseInt(_mapId);
            if (!this._allowedMapIds) {
                console.error("No basemaps found")
                return this._mapId
            } else if (this._allowedMapIds.includes(parsedMapId)) {
                return parsedMapId;
            } else {
                console.warn("Not a valid mapId");
                return this._mapId;
            }
        },

        getPlane: function () {
            return this._plane;
        },

        getMapId: function () {
            return this._mapId;
        },

        getMinPlane: function () {
            return this.options.minPlane || 0;
        },

        getMaxPlane: function () {
            return this.options.maxPlane || 3;
        },

        setMaxPlane: function (newMaxPlane) {
            this.options.maxPlane = newMaxPlane;
            this.fire('maxplanechange', {
                newMaxPlane: newMaxPlane
            });
        },

        setPlane: function (_plane) {
            let newPlane = this._limitPlane(_plane);
            let oldPlane = this._plane
                if (oldPlane !== newPlane) {
                    this.fire('preplanechange', {
                        oldPlane: oldPlane,
                        newPlane: newPlane
                    });
                    this.fire('viewprereset');
                    this._plane = newPlane;
                    this.fire('viewreset');
                    this.fire('planechange', {
                        oldPlane: oldPlane,
                        newPlane: newPlane
                    });
                    return this;
                }
        },

        setMapId: function (_mapId) {
            let newMapId = this._validateMapId(_mapId);
            let oldMapId = this._mapId
                if (oldMapId !== newMapId) {

                    this.fire('premapidchange', {
                        oldMapId: oldMapId,
                        newMapId: newMapId
                    });
                    this.fire('viewprereset');
                    this._mapId = newMapId;

                    this.fire('viewreset');
                    this.fire('mapidchange', {
                        oldMapId: oldMapId,
                        newMapId: newMapId
                    });
                    this.setMapIdBounds(newMapId);

                    return this;
                }
        },

        getMapIdBounds: function (mapId) {
            let[[west, south], [east, north]] = this._baseMaps[mapId].bounds;
            return L.latLngBounds([[south, west], [north, east]]);
        },

        setMapIdBounds: function (newMapId) {

            let bounds = this.getMapIdBounds(newMapId);

            if (this.options.showMapBorder) {
                this.boundsRect.setBounds(bounds);
            }

            let paddedBounds = bounds.pad(0.1);
            this.setMaxBounds(paddedBounds);

            this.fitWorld(bounds);
        },
    });

    L.gameMap = function (id, options) {
        return new L.GameMap(id, options);
    }

    L.TileLayer.Main = L.TileLayer.extend({
        initialize: function (url, options) {
            this._url = url;
            L.setOptions(this, options);
        },

        getTileUrl: function (coords) {
            return L.Util.template(this._url, {
                source: this.options.source,
                mapId: this._map._mapId,
                zoom: coords.z,
                plane: this._map._plane || 0,
                x: coords.x,
                y:  - (1 + coords.y),
            });
        },

        // Suppress 404 errors for loading tiles
        // These are expected as trivial tiles are not included to save on storage space
        createTile: function (coords, done) {
            let tile = L.TileLayer.prototype.createTile.call(this, coords, done);
            tile.onerror = error => error.preventDefault();
            return tile
        }

    });

    L.tileLayer.main = function (url, options) {
        return new L.TileLayer.Main(url, options);
    }

    L.Grid = L.GridLayer.extend({
        initialize: function (options) {
            options.maxNativeZoom = 2;
            options.minNativeZoom = 2;
            options.minZoom = 1;
            L.setOptions(this, options);
        },

        createTile: function (coords) {
            let tile = L.DomUtil.create('div', "grid");
            tile.innerHTML = `${coords.x}, ${- (1 + coords.y)}`;
            return tile;
        },

        _update: function (center) {
            if (this._map.getZoom() >= this.options.minZoom) {
                return L.GridLayer.prototype._update.call(this, center);
            }
        }
    });

    L.grid = function (options) {
        return new L.Grid(options);
    }
});
