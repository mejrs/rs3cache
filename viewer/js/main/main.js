'use strict';
const template = '../out/{source}/-1/{zoom}/{plane}_{x}_{y}.png';

void function (global) {
    let runescape_map = global.runescape_map = L.gameMap('map', {
        maxBounds: [[-1000, -1000], [12800 + 1000, 12800 + 1000]],
        maxBoundsViscosity: 0.5,
        customZoomControl: true,
        fullscreenControl: true,
        planeControl: true,
        positionControl: true,
        messageBox: true,
        initialMapId: -1,
        plane: 0,
        x: 3200,
        y: 3200,
        minPlane: 0,
        maxPlane: 3,
        minZoom: -4,
        maxZoom: 6,
        doubleClickZoom: false,
        showMapBorder: true,
        enableUrlLocation: true
    });

    let rs3 = L.tileLayer.main(template, {
        source: 'rs3/map_squares',
        minZoom: -4,
        maxNativeZoom: 4,
        maxZoom: 6,
    }).addTo(runescape_map);
	
	let osrs = L.tileLayer.main(template, {
        source: 'osrs/map_squares',
        minZoom: -4,
        maxNativeZoom: 2,
        maxZoom: 6,
    });

    let grid = L.grid({
        bounds: [[0, 0], [12800, 6400]],
    });

    L.control.layers.urlParam({
		"rs3": rs3,
		"osrs": osrs,
	}, {
        "grid": grid
    }, {
        collapsed: true,
        position: 'bottomright'
    }).addTo(runescape_map);

    // Check to see if the lumbridge tile is present
    let void_image = new Image;
    void_image.onerror = function () {
        runescape_map.addMessage("Could not load rs3 tiles. Please render maptiles first.", 60000)
    }
    void_image.src = '../out/rs3/map_squares/-1/1/0_25_25.png';
}
(this || window);
