use noise::{NoiseFn, Perlin};

fn calculate_biased_elevation(x: f64, y: f64, neighbors: &[(f64, f64)], map_width: f64, map_height: f64) -> f64 {

    let perlin = Perlin::new(2333);
    let mut sum_elevation = 0.0;
    let mut count = 0.0;

    // Calculate the elevation for the current tile
    let current_elevation = 75.0 * perlin.get([x / 100.0, y / 100.0]);
    sum_elevation += current_elevation;
    count += 1.0;

    // Calculate the elevation for each neighboring tile
    for (neighbor_x, neighbor_y) in neighbors {
        let neighbor_elevation = 75.0 * perlin.get([*neighbor_x / 100.0, *neighbor_y / 100.0]);
        sum_elevation += neighbor_elevation;
        count += 1.0;
    }

    // Calculate the average elevation
    let average_elevation = sum_elevation / count;

    // Calculate the distance from the center of the map
    let center_x = map_width / 2.0;
    let center_y = map_height / 2.0;
    let dx = x - center_x;
    let dy = y - center_y;
    let distance_from_center = (dx * dx + dy * dy).sqrt();

    // Calculate the maximum possible distance from the center of the map
    let max_distance = (center_x * center_x + center_y * center_y).sqrt();

    // Calculate the bias based on the distance from the center
    let bias = 1.0 - (distance_from_center / max_distance);

    // Apply the bias to the average elevation
    let biased_elevation = average_elevation * bias;

    biased_elevation
}

// calculate the elevation of a tile
fn calculate_elevation(x: f64, y: f64, width: i32, height: i32) -> i32 {
    let neighbors = [(x-1.0, y), (x+1.0, y), (x, y-1.0), (x, y+1.0)]; // left, right, up, down neighbors
    let elevation = calculate_biased_elevation(x as f64, y as f64, &neighbors, width as f64, height as f64);
    elevation as i32
}

fn make_tile( x: i32,y: i32, width:i32, height:i32) -> Tile {

    let elevation = calculate_elevation(x.into(), y.into(), width, height);
    let terrain_type = if elevation < 50 {
        TerrainType::Water
    } else if elevation < 100 {
        TerrainType::Grass
    } else {
        TerrainType::Mountain
    };

    Tile {
        terrain_type,
        x_position: x,
        y_position: y,
        elevation,
    }
}
