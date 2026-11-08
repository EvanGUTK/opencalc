//! Resolution-independent handheld artwork. Reference photos are only used in the guide.
use eframe::egui::{self, Align2, Color32 as C, FontId, Painter, Pos2, Rect, Stroke, Vec2};

const SHELL: C = C::from_rgb(29, 30, 29);
const BLUE: C = C::from_rgb(111, 188, 205);
const GREEN: C = C::from_rgb(162, 191, 116);

pub fn install_fonts(ctx: &egui::Context) {
    // Use the installed system typeface without bundling a licensed font with the app.
    let Some(windows) = std::env::var_os("WINDIR") else {
        return;
    };
    let path = std::path::PathBuf::from(windows).join("Fonts/arialbd.ttf");
    if let Ok(bytes) = std::fs::read(path) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "handheld-labels".into(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        let mut fallback = fonts.families[&egui::FontFamily::Proportional].clone();
        fallback.insert(0, "handheld-labels".into());
        fonts
            .families
            .insert(egui::FontFamily::Name("handheld-labels".into()), fallback);
        ctx.set_fonts(fonts);
        ctx.data_mut(|data| data.insert_temp(egui::Id::new("handheld-font"), true));
    }
}

pub fn draw_face(p: &Painter, face: Rect, scale: f32) {
    let contrast = high_contrast(p);
    let area = |x, y, w, h| {
        Rect::from_min_size(face.min + Vec2::new(x, y) * scale, Vec2::new(w, h) * scale)
    };
    for (x, y, w, h, radius, color) in [
        (8., 6., 307., 648., 41., C::from_gray(201)),
        (10., 3., 302., 647., 39., C::from_gray(74)),
        (12., 3., 296., 642., 36., SHELL),
    ] {
        p.rect_filled(
            area(x, y, w, h),
            (radius * scale).round() as u8,
            if contrast { C::BLACK } else { color },
        );
    }
    // A rounded mesh provides continuous plastic shading without raster artwork.
    let shade = |x: f32| {
        if contrast {
            return C::BLACK;
        }
        let highlight = 19. * (-((x - 17.) / 12.).powi(2)).exp();
        C::from_gray((28. + highlight + 2. * (1. - x / 323.)) as u8)
    };
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(area(161.5, 323., 0., 0.).min, shade(161.5));
    for (cx, cy, start) in [
        (48., 39., std::f32::consts::PI),
        (272., 39., std::f32::consts::PI * 1.5),
        (272., 607., 0.),
        (48., 607., std::f32::consts::FRAC_PI_2),
    ] {
        for i in 0..=20 {
            let angle = start + i as f32 / 20. * std::f32::consts::FRAC_PI_2;
            let pt = Vec2::new(cx, cy) + Vec2::angled(angle) * 35.;
            mesh.colored_vertex(face.min + pt * scale, shade(pt.x));
        }
    }
    let count = mesh.vertices.len() as u32;
    for i in 1..count {
        mesh.add_triangle(0, i, if i + 1 == count { 1 } else { i + 1 });
    }
    p.add(egui::Shape::mesh(mesh));
    for (x, y, w, h, radius, color) in [
        (30., 40., 262., 176., 22., C::from_gray(8)),
        (33., 43., 256., 170., 20., C::from_gray(17)),
        (50., 50., 220., 155., 2., C::from_gray(9)),
    ] {
        p.rect_filled(area(x, y, w, h), (radius * scale).round() as u8, color);
    }
    p.rect_stroke(
        area(13., 5., 293., 638.),
        (35. * scale) as u8,
        Stroke::new(
            if contrast { 2.0 * scale } else { scale },
            if contrast { C::WHITE } else { C::from_gray(47) },
        ),
        egui::StrokeKind::Inside,
    );
    // The two halves of the molded case meet just above the function keys.
    for (y, color) in [(224., C::from_gray(12)), (226., C::from_gray(53))] {
        p.line_segment(
            [area(13., y, 0., 0.).min, area(307., y, 0., 0.).min],
            Stroke::new(scale, color),
        );
    }
    bold_text(
        p,
        area(153., 23., 0., 0.).min,
        Align2::RIGHT_CENTER,
        "TI-84",
        FontId::proportional(14. * scale),
        C::WHITE,
        scale,
    );
    p.text(
        area(156., 23., 0., 0.).min,
        Align2::LEFT_CENTER,
        "Plus CE",
        FontId::proportional(12. * scale),
        C::from_gray(205),
    );
    bold_text(
        p,
        area(169., 631., 0., 0.).min,
        Align2::CENTER_CENTER,
        "TEXAS INSTRUMENTS",
        FontId::proportional(10. * scale),
        C::from_gray(218),
        scale * 0.7,
    );
    // Small molded emblem beside the maker's lettering.
    let emblem = [
        (96., 626.),
        (101., 626.),
        (102., 623.),
        (106., 626.),
        (109., 626.),
        (109., 631.),
        (113., 633.),
        (110., 637.),
        (106., 636.),
        (104., 631.),
        (100., 631.),
        (99., 634.),
        (96., 632.),
    ];
    p.add(egui::Shape::convex_polygon(
        emblem
            .iter()
            .map(|&(x, y)| area(x, y, 0., 0.).min)
            .collect(),
        C::from_gray(218),
        Stroke::NONE,
    ));

    // Four separate vector sectors keep the familiar directional pad crisp at any DPI.
    let center = area(233., 315., 0., 0.).min;
    p.circle_filled(center + Vec2::new(0., scale), 34. * scale, C::from_gray(10));
    for direction in 0..4 {
        let angle = direction as f32 * std::f32::consts::FRAC_PI_2;
        let mut sector = egui::Mesh::default();
        for i in 0..=24 {
            let a = angle - 0.67 + i as f32 / 24. * 1.34;
            let inner = 15.5;
            for radius in [inner, 32.] {
                sector.colored_vertex(center + Vec2::angled(a) * radius * scale, C::from_gray(226));
            }
            if i > 0 {
                let index = i * 2;
                sector.add_triangle(index - 2, index - 1, index);
                sector.add_triangle(index - 1, index + 1, index);
            }
        }
        p.add(egui::Shape::mesh(sector));
        let tip = center + Vec2::angled(angle) * 27. * scale;
        let base = center + Vec2::angled(angle) * 23. * scale;
        let side = Vec2::angled(angle + std::f32::consts::FRAC_PI_2) * 3. * scale;
        p.add(egui::Shape::convex_polygon(
            vec![tip, base + side, base - side],
            C::from_gray(70),
            Stroke::NONE,
        ));
    }
    let sun = area(229., 276., 0., 0.).min;
    p.circle_stroke(sun, 2. * scale, Stroke::new(scale, BLUE));
    for i in 0..8 {
        let ray = Vec2::angled(i as f32 * std::f32::consts::TAU / 8.);
        p.line_segment(
            [sun + ray * 3. * scale, sun + ray * 5. * scale],
            Stroke::new(scale, BLUE),
        );
    }
    p.circle_filled(sun + Vec2::new(11. * scale, 0.), 2.8 * scale, GREEN);
}

pub fn draw_key(p: &Painter, rect: Rect, key: &str, down: bool) {
    if key == "Learning, settings and accessibility"
        || key.starts_with("cursor ")
        || key == "previous calculation"
    {
        return;
    }
    let scale = rect.width() / 35.;
    let contrast = high_contrast(p);
    let number = key.chars().all(|c| c.is_ascii_digit()) || matches!(key, "." | "±");
    let light = number
        || matches!(
            key,
            "÷" | "×" | "−" | "+" | "enter" | "y=" | "window" | "zoom" | "trace" | "graph"
        );
    let color = match key {
        _ if contrast && light => C::WHITE,
        _ if contrast => C::BLACK,
        "2nd" => C::from_rgb(61, 169, 205),
        "alpha" => C::from_rgb(129, 182, 62),
        _ if light => C::from_rgb(230, 231, 228),
        _ => C::from_gray(28),
    };
    let function = matches!(key, "y=" | "window" | "zoom" | "trace" | "graph");
    let keycap = if function {
        rect.shrink2(Vec2::new(2. * scale, 2. * scale))
    } else if number {
        Rect::from_min_max(
            rect.min + Vec2::new(scale, 0.),
            rect.max + Vec2::new(-scale, 2. * scale),
        )
    } else {
        rect.shrink(scale)
    };
    let radius = (4. * scale).round() as u8;
    p.rect_filled(
        keycap.expand(1. * scale).translate(Vec2::new(0., scale)),
        radius + 2,
        C::from_gray(12),
    );
    let top = keycap.translate(Vec2::new(0., if down { scale } else { 0. }));
    p.rect_filled(
        top,
        radius,
        if down {
            color.gamma_multiply(0.78)
        } else {
            color
        },
    );
    p.rect_stroke(
        top,
        radius,
        Stroke::new(
            scale,
            if contrast {
                C::WHITE
            } else if light {
                C::from_gray(161)
            } else {
                C::from_gray(45)
            },
        ),
        egui::StrokeKind::Inside,
    );
    let label = match key {
        "delete" => "del",
        "x,t" => "X,T,θ,n",
        "1/x" => "x",
        "^" => "",
        "sto" => "sto",
        "±" => "(−)",
        _ => key,
    };
    let size = if number || matches!(key, "÷" | "×" | "−" | "+" | "^") {
        21.
    } else if matches!(key, "1/x" | "x²") {
        14.
    } else if key == "x,t" {
        8.
    } else if matches!(key, "window" | "graph" | "trace" | "enter") {
        8.5
    } else {
        11.5
    };
    bold_text(
        p,
        top.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(size * scale),
        if light {
            C::from_gray(26)
        } else {
            C::from_gray(239)
        },
        scale,
    );
    if key == "1/x" {
        bold_text(
            p,
            top.center() + Vec2::new(7. * scale, -4. * scale),
            Align2::CENTER_CENTER,
            "-1",
            FontId::proportional(7. * scale),
            C::from_gray(239),
            scale,
        );
    }
    if key == "^" {
        let center = top.center();
        p.add(egui::Shape::line(
            vec![
                center + Vec2::new(-10., 3.) * scale,
                center + Vec2::new(0., -5.) * scale,
                center + Vec2::new(10., 3.) * scale,
            ],
            Stroke::new(1.5 * scale, C::from_gray(239)),
        ));
    }
    if key == "vars" {
        p.text(
            Pos2::new(rect.right() - 10. * scale, y_legend(rect, scale)),
            Align2::CENTER_BOTTOM,
            "θ",
            FontId::proportional(8. * scale),
            if contrast { C::WHITE } else { GREEN },
        );
        p.circle_filled(
            Pos2::new(
                rect.right() - 2. * scale,
                y_legend(rect, scale) - 4. * scale,
            ),
            2.5 * scale,
            if contrast { C::WHITE } else { GREEN },
        );
    }
    if key == "sto" {
        let end = Pos2::new(top.right() - 3. * scale, top.center().y);
        p.arrow(
            end - Vec2::new(6. * scale, 0.),
            Vec2::new(6. * scale, 0.),
            Stroke::new(scale, C::WHITE),
        );
    }
    let (secondary, alpha) = legends(key);
    let y = rect.top() - 3. * scale;
    bold_text(
        p,
        Pos2::new(
            rect.left()
                - if key == "y=" {
                    14. * scale
                } else if function {
                    4. * scale
                } else {
                    0.
                },
            y,
        ),
        Align2::LEFT_BOTTOM,
        secondary,
        FontId::proportional(if key == "enter" { 7.0 } else { 8.5 } * scale),
        if contrast { C::WHITE } else { BLUE },
        scale * 0.65,
    );
    bold_text(
        p,
        Pos2::new(rect.right() + if function { 6. * scale } else { 0. }, y),
        Align2::RIGHT_BOTTOM,
        alpha,
        FontId::proportional(if key == "enter" { 7.0 } else { 8.3 } * scale),
        if contrast { C::WHITE } else { GREEN },
        scale * 0.65,
    );
}

fn bold_text(
    p: &Painter,
    pos: Pos2,
    align: Align2,
    text: &str,
    mut font: FontId,
    color: C,
    scale: f32,
) {
    if p.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("handheld-readable"))
            .unwrap_or(false)
    }) {
        font.family = egui::FontFamily::Monospace;
        font.size = font.size.max(9.5 * scale.max(0.9));
        p.text(pos, align, text, font, color);
        return;
    }
    if p.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("handheld-font"))
            .unwrap_or(false)
    }) {
        font.family = egui::FontFamily::Name("handheld-labels".into());
        p.text(pos, align, text, font, color);
        return;
    }
    // The bundled sans font is a light weight. Small subpixel offsets give printed legends
    // enough weight without depending on platform fonts or redistributing proprietary fonts.
    for offset in [-0.55, 0.0, 0.55] {
        p.text(
            pos + Vec2::new(offset * scale, 0.),
            align,
            text,
            font.clone(),
            color,
        );
    }
}

fn high_contrast(painter: &Painter) -> bool {
    painter.ctx().data(|data| {
        data.get_temp::<bool>(egui::Id::new("handheld-contrast"))
            .unwrap_or(false)
    })
}

fn y_legend(rect: Rect, scale: f32) -> f32 {
    rect.top() - 3. * scale
}

fn legends(key: &str) -> (&str, &str) {
    match key {
        "y=" => ("stat plot", "f1"),
        "window" => ("tblset", "f2"),
        "zoom" => ("format", "f3"),
        "trace" => ("calc", "f4"),
        "graph" => ("table", "f5"),
        "2nd" => ("", ""),
        "mode" => ("quit", ""),
        "delete" => ("ins", ""),
        "alpha" => ("A-lock", ""),
        "x,t" => ("link", ""),
        "stat" => ("list", ""),
        "math" => ("test", "A"),
        "apps" => ("angle", "B"),
        "prgm" => ("draw", "C"),
        "vars" => ("distr", ""),
        "clear" => ("", ""),
        "1/x" => ("matrix", "D"),
        "sin" => ("sin−¹", "E"),
        "cos" => ("cos−¹", "F"),
        "tan" => ("tan−¹", "G"),
        "^" => ("π", "H"),
        "x²" => ("√", "I"),
        "," => ("EE", "J"),
        "(" => ("{", "K"),
        ")" => ("}", "L"),
        "÷" => ("e", "M"),
        "log" => ("10^x", "N"),
        "7" => ("u", "O"),
        "8" => ("v", "P"),
        "9" => ("w", "Q"),
        "×" => ("[", "R"),
        "ln" => ("e^x", "S"),
        "4" => ("L4", "T"),
        "5" => ("L5", "U"),
        "6" => ("L6", "V"),
        "−" => ("]", "W"),
        "sto" => ("rcl", "X"),
        "1" => ("L1", "Y"),
        "2" => ("L2", "Z"),
        "3" => ("L3", "θ"),
        "+" => ("mem", "\""),
        "on" => ("off", ""),
        "0" => ("catalog", ""),
        "." => ("i", ":"),
        "±" => ("ans", "?"),
        "enter" => ("entry", "solve"),
        _ => ("", ""),
    }
}
