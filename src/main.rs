use tiny_skia::*;
use uniform_cubic_splines::*;

fn main() {
    let (width, height) = (256, 256);
    let mut pixmap = Pixmap::new(width, height).unwrap();

    let mut cvs = vec![
        (0.0f64, 0.0),
        (0.0, 0.0),
        (0.1, 0.1),
        (0.5, 0.8),
        (0.5, 0.8),
        (0.5, 0.8),
        (0.6, 0.3),
        (1.0, 1.0),
        (1.0, 1.0),
    ];

    let now = std::time::Instant::now();
    stroke_curve(&mut pixmap, &cvs, Basis::CatmullRom);

    println!(
        "Rendered in {:.2}ms",
        now.elapsed().as_micros() as f64 / 1000.0
    );

    pixmap.save_png("image.png").unwrap();
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
enum Basis {
    Linear,
    Hermite,
    CatmullRom,
}

impl From<Basis> for f64 {
    fn from(basis: Basis) -> Self {
        match basis {
            Basis::Linear => 0.0,
            Basis::Hermite => 0.25,
            Basis::CatmullRom => 0.5,
        }
    }
}

fn stroke_circle(x: f32, y: f32, r: f32) -> Path {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(x, y - r);
    path_builder.quad_to(x - r, y - r, x - r, y);
    path_builder.quad_to(x - r, y + r, x, y + r);
    path_builder.quad_to(x + r, y + r, x + r, y);
    path_builder.quad_to(x + r, y + r, x, y + r);
    path_builder.close();
    path_builder.finish().unwrap()
}

fn stroke_diamond(x: f32, y: f32, r: f32) -> Path {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(x - r, y);
    path_builder.line_to(x, y - r);
    path_builder.line_to(x + r, y);
    path_builder.line_to(x, y + r);
    path_builder.line_to(x - r, y);
    path_builder.close();
    path_builder.finish().unwrap()
}

struct CurveColors {}

fn stroke_curve(pixmap: &mut Pixmap, cvs: &Vec<(f64, f64)>, basis: Basis) {
    let mut curve_paint = Paint::default();
    curve_paint.set_color(Color::from_rgba(1.0, 1.0, 1.0, 1.0).unwrap());
    curve_paint.anti_alias = true;
    curve_paint.force_hq_pipeline = true;

    let mut dot_paint = Paint::default();
    dot_paint.set_color(Color::from_rgba(0.8, 0.8, 0.8, 1.0).unwrap());
    dot_paint.anti_alias = true;
    dot_paint.force_hq_pipeline = true;

    let mut dot_fill = Paint::default();
    dot_fill.set_color(Color::from_rgba(0.2, 0.2, 0.2, 1.0).unwrap());
    dot_fill.anti_alias = false;

    let mut curve_stroke = Stroke::default();
    curve_stroke.width = 4.0;

    let mut dot_stroke = Stroke::default();
    dot_stroke.width = 2.0;

    let path = {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(
            0f32,
            (1.0 - cvs[1].1) as f32 * (pixmap.height() as f32).floor() + 0.5,
        );

        if Basis::Linear == basis {
            cvs.iter()
                .for_each(|cv| path_builder.line_to(cv.0 as f32, cv.1 as f32));
        } else {
            let knots = cvs.iter().map(|cv| cv.0).collect::<Vec<_>>();
            let points = cvs.iter().map(|cv| cv.1).collect::<Vec<_>>();

            let segments = (pixmap.width() as usize >> 3) * (cvs.len() - 3);

            let step = 1.0 / segments as f64;
            let mut x = 0.0f64;
            for i in 0..segments {
                let v = spline_inverse::<basis::CatmullRom, _>(x, &knots).unwrap();

                path_builder.line_to(
                    (x * pixmap.width() as f64) as _,
                    (1.0 - spline::<basis::CatmullRom, _, _>(v, &points)) as f32
                        * pixmap.height() as f32,
                );
                x += step;
            }
        }

        path_builder.finish().unwrap()
    };

    pixmap.stroke_path(&path, &curve_paint, &curve_stroke, Transform::identity(), None);

    cvs.iter().for_each(|cv| {
        let handle_path = PathBuilder::from_circle(
            (cv.0 * pixmap.width() as f64) as _,
            ((1.0 - cv.1) * pixmap.height() as f64) as _,
            5.5,
        )
        .unwrap();
        /*
        let diamond_path = stroke_diamond(
            cv.0 as f32,
            cv.1 as f32,
            6.0,
        );*/
        pixmap.fill_path(&handle_path, &dot_fill, FillRule::Winding, Transform::identity(), None);
        pixmap.stroke_path(&handle_path, &dot_paint, &dot_stroke, Transform::identity(), None);
    });
}
