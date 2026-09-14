mod sys;

use alloc::ffi::CString;

use core::{
    ffi::{c_int, CStr},
    marker::PhantomData,
    ops::{
        Add,
        AddAssign,
        Mul,
        MulAssign,
        Sub,
        SubAssign,
    },
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
};
// ============================================================
// Math
// ============================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }

    pub fn deadzone(self, radius: f32) -> Self {
        let length_squared =
            self.x * self.x +
                self.y * self.y;

        if length_squared < radius * radius {
            Self::ZERO
        } else {
            self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
        )
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
        )
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(
            self.x * rhs,
            self.y * rhs,
        )
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Vec2::new(x, y)
    }
}

impl From<(usize, usize)> for Vec2 {
    fn from((x, y): (usize, usize)) -> Self {
        Vec2::new(x as f32, y as f32)
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from((x, y): (i32, i32)) -> Self {
        Vec2::new(x as f32, y as f32)
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}


// ============================================================
// Rectangle
// ============================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn new_usize(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            width: width as f32,
            height: height as f32,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Rect {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Rect::new(value.0, value.1, value.2, value.3)
    }
}

impl From<(usize, usize, usize, usize)> for Rect {
    fn from(value: (usize, usize, usize, usize)) -> Self {
        Rect::new_usize(value.0, value.1, value.2, value.3)
    }
}


// ============================================================
// Color
// ============================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub const fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from * (1.0 - t) + to * t
}

pub const fn lerp_u8(from: u8, to: u8, t: f32) -> u8 {
    lerp(from as f32, to as f32, t) as u8
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const fn rgba(
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> Self {
        Self { r, g, b, a }
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Self { r: self.r, g: self.g, b: self.b, a}
    }

    pub const fn darken(self, amount: u8) -> Self {
        Self {
            r: self.r.saturating_sub(amount),
            g: self.g.saturating_sub(amount),
            b: self.b.saturating_sub(amount),
            a: self.a
        }
    }

    /// Tints the color by a percentage (0 -> 1),
    /// where 0 is base color and 1 means fully tint_color.
    /// Doesn't affect alpha
    pub const fn tint(self, tint_color: Color, amount: f32) -> Self {
        Self {
            r: lerp_u8(self.r, tint_color.r, amount),
            g: lerp_u8(self.g, tint_color.g, amount),
            b: lerp_u8(self.b, tint_color.b, amount),
            a: self.a
        }
    }

    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    pub const RED: Self =
        Self::rgb(230, 41, 55);

    pub const GREEN: Self =
        Self::rgb(0, 228, 48);

    pub const BLUE: Self =
        Self::rgb(0, 121, 241);

    pub const YELLOW: Self =
        Self::rgb(253, 249, 0);

    pub const SKY_BLUE: Self =
        Self::rgb(102, 191, 255);

    pub const RAY_WHITE: Self =
        Self::rgb(245, 245, 245);
}

// ============================================================
// Input
// ============================================================

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Up = 1,
    Right = 2,
    Down = 3,
    Left = 4,

    X = 5,
    A = 6,
    B = 7,
    Y = 8,

    L = 9,
    ZL = 10,
    R = 11,
    ZR = 12,

    Minus = 13,
    Plus = 15,

    LeftStick = 16,
    RightStick = 17,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    LeftX = 0,
    LeftY = 1,
    RightX = 2,
    RightY = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct Gamepad {
    id: i32,
}

impl Gamepad {
    pub fn new() -> Self { Self::new_by_id(0) }
    pub fn new_by_id(id: i32) -> Self { Self { id } }

    pub fn available(self) -> bool {
        unsafe {
            sys::IsGamepadAvailable(self.id)
        }
    }

    pub fn pressed(self, button: Button) -> bool {
        unsafe {
            sys::IsGamepadButtonPressed(
                self.id,
                button as i32,
            )
        }
    }

    pub fn down(self, button: Button) -> bool {
        unsafe {
            sys::IsGamepadButtonDown(
                self.id,
                button as i32,
            )
        }
    }

    pub fn released(self, button: Button) -> bool {
        unsafe {
            sys::IsGamepadButtonReleased(
                self.id,
                button as i32,
            )
        }
    }

    pub fn axis(self, axis: Axis) -> f32 {
        unsafe {
            sys::GetGamepadAxisMovement(
                self.id,
                axis as i32,
            )
        }
    }

    /// Screen-oriented stick:
    ///
    /// +X = right
    /// +Y = down
    pub fn left_stick(self) -> Vec2 {
        Vec2::new(
            self.axis(Axis::LeftX),
            -self.axis(Axis::LeftY),
        )
    }

    /// Screen-oriented stick:
    ///
    /// +X = right
    /// +Y = down
    pub fn right_stick(self) -> Vec2 {
        Vec2::new(
            self.axis(Axis::RightX),
            -self.axis(Axis::RightY),
        )
    }
}


// ============================================================
// App
// ============================================================

static APP_EXISTS: AtomicBool =
    AtomicBool::new(false);

pub struct App {
    // Prevent App from being Send/Sync.
    // The raylib window/context is treated as thread-affine.
    _not_send_sync: PhantomData<*mut ()>,
}

#[derive(Debug, Clone, Copy)]
pub enum AppError {
    AlreadyInitialized,
    WindowInitializationFailed,
}

impl App {
    pub fn new(
        width: usize,
        height: usize,
        title: &str,
    ) -> Result<Self, AppError> {
        if APP_EXISTS.swap(true, Ordering::Relaxed) {
            return Err(AppError::AlreadyInitialized);
        }

        let c_str = CString::new(title).unwrap();

        unsafe {
            sys::InitWindow(
                width as c_int,
                height as c_int,
                c_str.as_ptr(),
            );
        }

        if !unsafe { sys::IsWindowReady() } {
            APP_EXISTS.store(
                false,
                Ordering::Relaxed,
            );

            return Err(
                AppError::WindowInitializationFailed
            );
        }

        Ok(Self {
            _not_send_sync: PhantomData,
        })
    }

    pub fn set_target_fps(&mut self, fps: i32) {
        unsafe {
            sys::SetTargetFPS(fps);
        }
    }

    pub fn running(&self) -> bool {
        !unsafe {
            sys::WindowShouldClose()
        }
    }

    pub fn delta_time(&self) -> f32 {
        unsafe {
            sys::GetFrameTime()
        }
    }

    pub fn begin_frame(
        &mut self,
        clear: Color,
    ) -> Frame<'_> {
        unsafe {
            sys::BeginDrawing();
            sys::ClearBackground(clear);
        }

        Frame {
            _app: PhantomData,
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            sys::CloseWindow();
        }

        APP_EXISTS.store(
            false,
            Ordering::Relaxed,
        );
    }
}


// ============================================================
// Frame
// ============================================================

pub struct Frame<'app> {
    // Makes Rust think we are borrowing an app when we create a frame
    // This prevents us from creating another frame before we drop previous one
    _app: PhantomData<&'app mut App>,
}

impl Frame<'_> {
    pub fn line(
        &mut self,
        from: Vec2,
        to: Vec2,
        thickness: f32,
        color: Color,
    ) {
        unsafe {
            sys::DrawLineEx(
                from,
                to,
                thickness,
                color,
            );
        }
    }

    pub fn rect(
        &mut self,
        rect: Rect,
        color: Color,
    ) {
        unsafe {
            sys::DrawRectangleRec(
                rect,
                color,
            );
        }
    }

    /// This is different from plain DrawRectanglePro call, because origin is set via
    /// applying relative coordinates (i.e. 0 -> 1), instead of raw pixel values
    pub fn rect_rotation(
        &mut self,
        rect: Rect,
        origin: Vec2,
        rotation: f32,
        color: Color,
    ) {
        let origin = Vec2::new(origin.x * rect.width, origin.y * rect.height);

        unsafe {
            sys::DrawRectanglePro(
                rect,
                origin,
                rotation,
                color,
            )
        }
    }

    pub fn circle(
        &mut self,
        center: Vec2,
        radius: f32,
        color: Color,
    ) {
        unsafe {
            sys::DrawCircleV(
                center,
                radius,
                color,
            );
        }
    }

    /// Convenient version.
    ///
    /// Allocates a temporary NUL-terminated buffer.
    pub fn text(
        &mut self,
        text: &str,
        position: Vec2,
        size: i32,
        color: Color,
    ) {
        let c_str = CString::new(text).unwrap();

        self.const_text(
            &c_str,
            position,
            size,
            color,
        );
    }

    /// Allocation-free version for persistent/static text.
    pub fn const_text(
        &mut self,
        text: &CStr,
        position: Vec2,
        size: i32,
        color: Color,
    ) {
        unsafe {
            sys::DrawText(
                text.as_ptr(),
                position.x as i32,
                position.y as i32,
                size,
                color,
            );
        }
    }

    pub fn measure_text(
        &self,
        text: &str,
        size: i32,
    ) -> i32 {
        let c_str = CString::new(text).unwrap();

        self.measure_const_text(&c_str, size)
    }

    pub fn measure_const_text(
        &self,
        text: &CStr,
        size: i32,
    ) -> i32 {
        unsafe {
            sys::MeasureText(
                text.as_ptr(),
                size as c_int,
            )
        }
    }

    pub fn text_right_align(
        &mut self,
        text: &str,
        size: i32,
        right_align_border: usize,
        horizontal_offset: i32,
        y: i32,
        color: Color,
    ) {
        let c_str = CString::new(text).unwrap();

        self.const_text_right_align(&c_str, size, right_align_border, horizontal_offset, y, color);
    }

    pub fn const_text_right_align(
        &mut self,
        text: &CStr,
        size: i32,
        right_align_border: usize,
        horizontal_offset: i32,
        y: i32,
        color: Color,
    ) {
        let x = right_align_border as i32 - self.measure_const_text(&text, size) - horizontal_offset;

        self.const_text(&text, (x, y).into(), size, color)
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::EndDrawing();
        }
    }
}

// ============================================================
// Random
// ============================================================
pub fn get_random_value(min: i32, max: i32) -> i32 {
    unsafe {
        sys::GetRandomValue(min, max)
    }
}