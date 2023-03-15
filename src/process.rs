use opencv::{core, highgui, imgproc, prelude::*};
use std::cmp::Ordering;
use std::f64;

const DISTANCE_THRESHOLD: f64 = 20.0;
const RADIUS: i32 = 11;

#[derive(Default)]
struct Enemy {
    last_seen: i32,
    existence: i32,
    coord: (i32, i32),
    pic: core::Mat,
}

fn new_enemy(coords: (i32, i32), cropped: &core::Mat, data: &mut Vec<Enemy>) {
    println!("new enemy");
    let enemy = Enemy {
        coord: coords,
        pic: cropped.clone().unwrap(),
        ..Default::default()
    };
    data.push(enemy);
}

fn refresh(enemy: &mut Enemy, coord: (i32, i32)) {
    enemy.coord = coord;
    enemy.last_seen = 0;
    enemy.existence += 1;
}

fn nearest(coord: (i32, i32), data: &Vec<Enemy>) -> Option<&Enemy> {
    let mut nearest: Option<&Enemy> = None;
    let mut nearest_distance = f64::INFINITY;

    for enemy in data.iter() {
        let d = distance(enemy.coord, coord);
        if d < nearest_distance {
            nearest = Some(enemy);
            nearest_distance = d;
        }
    }

    if nearest_distance > DISTANCE_THRESHOLD {
        nearest = None;
    }

    nearest
}

fn distance(co1: (i32, i32), co2: (i32, i32)) -> f64 {
    ((co1.0 - co2.0).pow(2) as f64 + (co1.1 - co2.1).pow(2) as f64).sqrt()
}

fn compare_hist(img1: &core::Mat, img2: &core::Mat) -> f64 {
    let h1 = {
        let mut h = core::Mat::default();
        let channels = vec![0];
        let hist_size = vec![256];
        let ranges = vec![0.0, 256.0];
        let ranges_vec = vec![ranges.clone()];
        imgproc::calc_hist(
            &vec![img1],
            &channels,
            &core::Mat::default(),
            &mut h,
            &hist_size,
            &ranges_vec,
            false,
        )
            .unwrap();
        h
    };

    let h2 = {
        let mut h = core::Mat::default();
        let channels = vec![0];
        let hist_size = vec![256];
        let ranges = vec![0.0, 256.0];
        let ranges_vec = vec![ranges.clone()];
        imgproc::calc_hist(
            &vec![img2],
            &channels,
            &core::Mat::default(),
            &mut h,
            &hist_size,
            &ranges_vec,
            false,
        )
            .unwrap();
        h
    };

    let compare = imgproc::compare_hist(&h1, &h2, imgproc::HISTCMP_CORREL).unwrap();
    println!("compare: {}", compare);

    compare
}


fn process(image: &mut core::Mat, data: &mut Vec<Enemy>) -> core::Mat {
    /*
     * An image of minimap ---> show last seen position of champions
     */

    let mut coords = vec![];

    let mut bgr = {
        let mut bgr = core::Mat::default();
        imgproc::cvt_color(image, &mut bgr, imgproc::COLOR_BGR2RGB, 0).unwrap();
        bgr
    };

    let mut in_range_r = core::Mat::default();
    let mut in_range_g = core::Mat::default();
    let mut in_range_b = core::Mat::default();
    let mut induction = core::Mat::default();
    imgproc::in_range(&bgr, &core::Scalar::new(120.0, 0.0, 0.0, 0.0), &core::Scalar::new(255.0, 0.0, 0.0, 0.0), &mut in_range_r).unwrap();
    imgproc::in_range(&bgr, &core::Scalar::new(0.0, 120.0, 0.0, 0.0), &core::Scalar::new(0.0, 255.0, 0.0, 0.0), &mut in_range_g).unwrap();
    imgproc::in_range(&bgr, &core::Scalar::new(0.0, 0.0, 120.0, 0.0), &core::Scalar::new(0.0, 0.0, 255.0, 0.0), &mut in_range_b).unwrap();
    core::subtract(&in_range_r, &in_range_g, &mut induction, &core::Mat::default()).unwrap();
    core::subtract(&induction, &in_range_b, &mut induction, &core::Mat::default()).unwrap();

// regarder la map et detecter les ennemis
    let circles = {
        let mut circles = core::Mat::default();
        imgproc::HoughCircles(
            &induction,
            &mut circles,
            imgproc::HOUGH_GRADIENT,
            1.0,
            10.0,
            30.0,
            15.0,
            9,
            30,
        )
            .unwrap();
        circles
    };

    if let Some(circles) = circles {
        for n in 0..circles.rows().unwrap() {
            let coord = (
                circles.at::<f32>(n, 0).unwrap() as i32,
                circles.at::<f32>(n, 1).unwrap() as i32,
            );

            let near = nearest(coord, data);
            if let Some(mut near_enemy) = near {
                if near_enemy.last_seen < 5 {
                    refresh(&mut near_enemy, coord);
                }
            } else {
                let cropped = {
                    let mut cropped = core::Mat::default();
                    let rect = core::Rect::new(coord.0 - RADIUS, coord.1 - RADIUS, 2 * RADIUS, 2 * RADIUS);
                    let mut cropped_tmp = bgr.region(rect).unwrap();
                    imgproc::resize(
                        &cropped_tmp,
                        &mut cropped,
                        core::Size::new(24, 24),
                        0.0,
                        0.0,
                        imgproc::INTER_LINEAR,
                    )
                        .unwrap();
                    cropped
                };

                if data.len() < 5 {
                    new_enemy(coord, &cropped, data);
                } else {
                    let similar = data.iter().max_by(|x, y| {
                        let a = compare_hist(&x.pic, &cropped);
                        let b = compare_hist(&y.pic, &cropped);
                        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
                    });
                    if let Some(similar_enemy) = similar {
                        println!(
                            "
best match : {}",
                            compare_hist(&similar_enemy.pic, &cropped)
                        );
                        refresh(&mut similar_enemy.clone(), coord);
                    }
                }
            }
        }
    }// si un ennemi n'est pas detecte depuis longtemps
    data.retain(|mut enemy| {
        if enemy.last_seen > 5 {
            if enemy.existence < 4 {
                false
            } else if enemy.last_seen > 666 {
                true
            } else {
                let pic_bw_resized = {
                    let mut pic_bw = core::Mat::default();
                    imgproc::cvt_color(&enemy.pic, &mut pic_bw, imgproc::COLOR_BGR2GRAY, 0).unwrap();
                    let mut pic_bw_resized = core::Mat::default();
                    imgproc::resize(
                        &pic_bw,
                        &mut pic_bw_resized,
                        core::Size::new(RADIUS, RADIUS),
                        0.0,
                        0.0,
                        imgproc::INTER_LINEAR,
                    )
                        .unwrap();
                    pic_bw_resized
                };

                let rect = core::Rect::new(
                    enemy.coord.0 - RADIUS + 5,
                    enemy.coord.1 - RADIUS + 5,
                    RADIUS,
                    RADIUS,
                );
                let mut submat = bgr.region_mut(rect).unwrap();
                imgproc::cvt_color(
                    &pic_bw_resized,
                    &mut submat,
                    imgproc::COLOR_GRAY2BGR,
                    0,
                ).unwrap();

                // cv2.rectangle(image, (n.coord[0] - radius, n.coord[1] - radius),
                //               (n.coord[0] + radius, n.coord[1] + radius), (0, 0, 255), 1)

                enemy.last_seen += 1;
                true
            }
        } else {
            enemy.last_seen += 1;
            true
        }
    });

    bgr
}

fn main() {
    let mut data: Vec<Enemy> = vec![];
    let window = "Rust OpenCV Example";
    highgui::named_window(window, highgui::WINDOW_NORMAL).unwrap();
    let mut frame = highgui::imread("minimap.png", highgui::IMREAD_COLOR).unwrap();

    loop {
        let result = process(&mut frame, &mut data);
        highgui::imshow(window, &result).unwrap();

        let key = highgui::wait_key(10).unwrap();
        if key > 0 && key != 255 {
            break;
        }
    }
}
