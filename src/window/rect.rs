

#[derive(Clone)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x: x.try_into().expect("X point can not be more than u16 value"),
            y: y.try_into().expect("Y point can not be more than u16 value")
        }
    }

    pub fn buf_addr(&self, screen_width: u16) -> usize {
        (self.y * screen_width + self.x).into()
    }

    pub fn set(&mut self, x: u16, y: u16) -> &Self {
        self.x += x;
        self.y += y;
        self
    }

    pub fn add(&self, x: u16, y: u16) -> Self {
        Point::new((self.x + x).into(), (self.y + y).into())
    }
}

struct RectPadd {
    top: u16,
    right: u16,
    bottom: u16, 
    left: u16,
}

impl Default for RectPadd {
    fn default() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }
}

impl ToString for RectPadd {
    fn to_string(&self) -> String {
        format!("{}, {}, {}, {}", self.top, self.right, self.bottom, self.left)
    }
}

pub struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16, 
    padding: RectPadd,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { 
            x, 
            y, 
            width, 
            height,
            padding: RectPadd::default(),
        } 
    }

    pub fn set_padding(
        &mut self, 
        top: Option<u16>, 
        right: Option<u16>, 
        bottom: Option<u16>, 
        left: Option<u16>
    ) {
        if let Some(top) = top {
            self.padding.top = top;
        }
        if let Some(right) = right {
            self.padding.right = right;
        }
        if let Some(bottom) = bottom {
            self.padding.bottom = bottom;
        }
        if let Some(left) = left {
            self.padding.left = left;
        }
    }

    pub fn anchor(&self) -> Point {
        Point::new(self.x.into(), self.y.into())
    }
    
    pub fn set_anchor(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn set_dimensions(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;
    }

    pub fn top_left(&self) -> Point {
        Point::new(self.x.into(), self.y.into())
    }

    pub fn top_left_padded(&self) -> Point {
        let x = (self.x + self.padding.left).into();
        let y = (self.y + self.padding.top).into();
        Point::new(x, y)
    }

    pub fn top_right(&self) -> Point {
        let x = self.x + self.width;
        Point::new(x.into(), self.y.into())
    }

    pub fn top_right_padded(&self) -> Point {
        let x = (self.width - self.padding.right).into();
        let y = (self.y + self.padding.top).into();
        Point::new(x, y)
    }

    pub fn bottom_left(&self) -> Point {
        let y = self.y + self.height;
        Point::new(self.x.into(), y.into())
    }

    pub fn bottom_left_padded(&self) -> Point {
        let x = (self.x + self.padding.left).into();
        let y = (self.y + self.height - self.padding.bottom).into();
        Point::new(x, y)
    }


    pub fn bottom_right(&self) -> Point {
        let x = self.x + self.width;
        let y = self.y + self.height;
        Point::new(x.into(), y.into())
    }
     
    pub fn width(&self) -> u16 {
        self.width 
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn point(&self, x: u16, y: u16) -> Point {
        let x = self.x + x;
        let y = self.y + y;
        Point::new(x.into(), y.into())
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            x: 0, 
            y: 0, 
            width: 0, 
            height: 0,
            padding: RectPadd::default(),
        }
    }
}
