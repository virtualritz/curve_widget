use cubic_spline::*;
use tiny_skia::*;

fn main() {

    let (width, height) = (256, 256);
    let mut canvas = Canvas::from(Pixmap::new(width, height).unwrap());

    let mut cvs = vec![
        (0.0f64, 0.0),
        (0.0, 0.0),
        (0.3, 0.1),
        (0.5, 0.8),
        (1.0, 1.0),
        (1.0, 1.0),
    ];

    cvs.iter_mut().for_each(|cv| {
        cv.0 = (cv.0 * canvas.pixmap.width() as f64).floor() + 0.5;
        cv.1 = ((1.0 - cv.1) * canvas.pixmap.height() as f64).floor() + 0.5;

        println!("{:?}", cv);
    });

    let now = std::time::Instant::now();
    stroke_curve(&mut canvas, &cvs, Basis::CatmullRom);

    println!(
        "Rendered in {:.2}ms",
        now.elapsed().as_micros() as f64 / 1000.0
    );

    canvas.pixmap.save_png("image.png").unwrap();
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

fn stroke_curve(canvas: &mut Canvas, cvs: &Vec<(f64, f64)>, basis: Basis) {
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
        path_builder.move_to(cvs.first().unwrap().0 as f32, cvs.first().unwrap().1 as f32);

        if Basis::Linear == basis {
            cvs.iter()
                .for_each(|cv| path_builder.line_to(cv.0 as f32, cv.1 as f32));
        } else {
            Spline::from_tuples(
                cvs,
                &SplineOpts {
                    num_of_segments: canvas.pixmap.width() >> 3,
                    tension: basis.into(),
                    ..Default::default()
                },
            )
            .iter()
            .for_each(|cv| path_builder.line_to(cv.0 as f32, cv.1 as f32));
        }

        path_builder.finish().unwrap()
    };

    let stroked_path = path.stroke(curve_stroke).unwrap();

    canvas.fill_path(&stroked_path, &curve_paint, FillType::Winding);

    cvs.iter().for_each(|cv| {
        let handle_path = PathBuilder::from_circle(cv.0 as f32, cv.1 as f32, 5.5).unwrap();
        /*
        let diamond_path = stroke_diamond(
            cv.0 as f32,
            cv.1 as f32,
            6.0,
        );*/
        canvas.fill_path(&handle_path, &dot_fill, FillType::Winding);
        canvas.fill_path(&handle_path.stroke(dot_stroke).unwrap(), &dot_paint, FillType::Winding);
    });
}
