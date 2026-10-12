//! GDI 整句译文的换行测量与绘制，保持 DPI 缩放后的宽度限制。

use qingjian_render::Row;
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    DT_CALCRECT, DT_NOPREFIX, DT_WORDBREAK, DrawTextW, HDC, HFONT, SelectObject, SetTextColor,
};

use super::theme::Theme;

pub(super) fn text(row: &Row) -> String {
    row.annotation
        .iter()
        .map(|(text, _)| text.as_str())
        .collect()
}

pub(super) fn width(theme: &Theme, available: i32) -> i32 {
    // padding 是 8 DIP；与位图渲染器的 280–560 点范围一致。
    available.clamp(theme.padding * 35, theme.padding * 70)
}

pub(super) fn measure(hdc: HDC, font: HFONT, text: &str, width: i32) -> SIZE {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    let mut rect = RECT {
        right: width,
        ..RECT::default()
    };
    unsafe {
        SelectObject(hdc, font.into());
        DrawTextW(
            hdc,
            &mut utf16,
            &mut rect,
            DT_CALCRECT | DT_NOPREFIX | DT_WORDBREAK,
        );
    }
    SIZE {
        cx: rect.right - rect.left,
        cy: rect.bottom - rect.top,
    }
}

pub(super) fn draw(hdc: HDC, font: HFONT, color: COLORREF, text: &str, rect: RECT) {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    let mut rect = rect;
    unsafe {
        SelectObject(hdc, font.into());
        SetTextColor(hdc, color);
        DrawTextW(hdc, &mut utf16, &mut rect, DT_NOPREFIX | DT_WORDBREAK);
    }
}
