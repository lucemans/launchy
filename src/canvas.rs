use crate::Color;


pub trait Canvas {
	const BOUNDING_BOX_WIDTH: u32;
	const BOUNDING_BOX_HEIGHT: u32;

	// These are the methods that _need_ to be implemented by the.. implementor

	/// Check if the location is in bounds
	fn is_valid(x: u32, y: u32) -> bool;
	/// Retrieves the current color at the given location. No bounds checking
	fn get_unchecked(&self, x: u32, y: u32) -> Color;
	/// Sets the color at the given location. No bounds checking
	fn set_unchecked(&mut self, x: u32, y: u32, color: Color);
	/// Retrieves the old, unflushed color at the given location. No bounds checking
	fn get_old_unchecked(&self, x: u32, y: u32) -> Color;
	/// Flush the accumulated changes to the underlying device
	fn flush(&mut self) -> anyhow::Result<()>;
	
	// These are defaut implementations that you get for free

	/// Sets the color at the given location. Panics if the location is out of bounds
	fn set(&mut self, x: u32, y: u32, color: Color) {
		if !Self::is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		self.set_unchecked(x, y, color);
	}

	/// Sets the color at the given location. Panics if the location is out of bounds
	fn get(&self, x: u32, y: u32) -> Color {
		if !Self::is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		return self.get_unchecked(x, y);
	}

	/// Retrieves the old, unflushed color at the given location. Panics if the location is out of
	/// bounds
	fn get_old(&self, x: u32, y: u32) -> Color {
		if !Self::is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		return self.get_old_unchecked(x, y);
	}

	fn iter() -> CanvasIterator<Self> {
		return CanvasIterator::new();
	}

	// fn iter_mut(&mut self) -> CanvasIteratorMut<Self> {
	// 	return CanvasIteratorMut::new(self);
	// }
}

// Next lines are canvas iteration stuff...

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CanvasButton<C: Canvas + ?Sized> {
	// canvas button coordinates MUST be valid!
	x: u32,
	y: u32,

	// we need to restrict ourselves to just this specific canvas type, because the coordinates may
	// be invalid for other canvas types
	phantom: std::marker::PhantomData<C>,
}

impl<C: Canvas + ?Sized> CanvasButton<C> {
	pub fn x(&self) -> u32 { self.x }
	pub fn y(&self) -> u32 { self.y }

    pub fn get(&self, canvas: &C) -> Color {
		canvas.get_unchecked(self.x, self.y)
	}

    pub fn get_old(&self, canvas: &C) -> Color {
		canvas.get_old_unchecked(self.x, self.y)
	}

	pub fn set(&self, canvas: &mut C, color: Color) {
		canvas.set_unchecked(self.x, self.y, color);
	}
}

pub struct CanvasIterator<C: Canvas + ?Sized> {
	// These are on a valid state at the start, and right before the next valid state afterwards
	x: u32,
	y: u32,

	phantom: std::marker::PhantomData<C>, // dunno why rustc needs this but whatever
}

impl<C: Canvas + ?Sized> CanvasIterator<C> {
	fn new() -> Self {
		let mut iter = CanvasIterator {
			x: 0,
			y: 0,
			phantom: std::marker::PhantomData,
		};
		iter.find_next_valid(); // get to a valid state
		return iter;
	}

	fn advance(&mut self) {
		self.x += 1;
		if self.x == C::BOUNDING_BOX_WIDTH {
			self.x = 0;
			self.y += 1;
		}
	}

	// Returns false if there is no more valid state to go to
	fn find_next_valid(&mut self) -> bool {
		loop {
			if self.y >= C::BOUNDING_BOX_HEIGHT { return false }
			if C::is_valid(self.x, self.y) { return true }
			// if the current position is not out of bounds but still invalid, let's continue
			// searching
			self.advance();
		}
	}
}

impl<C: Canvas> Iterator for CanvasIterator<C> {
	type Item = CanvasButton<C>;

	fn next(&mut self) -> Option<Self::Item> {
		let in_bounds = self.find_next_valid();
		if !in_bounds { return None };

		let value = CanvasButton {
			x: self.x,
			y: self.y,
			phantom: std::marker::PhantomData,
		};

		self.advance();

		return Some(value);
	}
}

/*// Wow that was a lot of code for canvas iteration. Let's just..... do it all again (:
// I need to repeat all the code in order to have a mutable version.. ugh

pub struct CanvasButtonMut<'a, C: Canvas + ?Sized> {
	canvas: *mut C,
	// canvas button coordinates MUST be valid!
	x: u32,
	y: u32,
	phantom: std::marker::PhantomData<&'a C>,
}

impl<'a, C: Canvas + ?Sized> CanvasButtonMut<'a, C> {
	pub fn x(&self) -> u32 { self.x }
	pub fn y(&self) -> u32 { self.y }

    pub fn get(&self) -> Color {
		unsafe {
			return (*self.canvas).get_unchecked(self.x, self.y);
		}
	}

    pub fn get_old(&self) -> Color {
		unsafe {
			return (*self.canvas).get_old_unchecked(self.x, self.y);
		}
	}
	
    pub fn set(&mut self, color: Color) {
		unsafe {
			return (*self.canvas).set_unchecked(self.x, self.y, color);
		}
	}
}

pub struct CanvasIteratorMut<'a, C: Canvas + ?Sized> {
	canvas: &'a mut C,
	// These are on a valid state at the start, and right before the next valid state afterwards
	x: u32,
	y: u32,
}

impl<'a, C: Canvas + ?Sized> CanvasIteratorMut<'a, C> {
	fn new(canvas: &'a mut C) -> Self {
		let mut iter = CanvasIteratorMut {
			canvas,
			x: 0,
			y: 0,
		};
		iter.find_next_valid(); // get to a valid state
		return iter;
	}

	fn advance(&mut self) {
		self.x += 1;
		if self.x == C::BOUNDING_BOX_WIDTH {
			self.y += 1;
		}
	}

	// Returns false if there is no more valid state to go to
	fn find_next_valid(&mut self) -> bool {
		loop {
			if self.y >= C::BOUNDING_BOX_HEIGHT { return false }
			if C::is_valid(self.x, self.y) { return true }
			// if the current position is not out of bounds but still invalid, let's continue
			// searching
			self.advance();
		}
	}
}

impl<'a, C: Canvas> Iterator for CanvasIteratorMut<'a, C> {
	type Item = CanvasButtonMut<'a, C>;

	fn next(&mut self) -> Option<Self::Item> {
		let in_bounds = self.find_next_valid();
		if !in_bounds { return None };

		let value = CanvasButtonMut {
			canvas: self.canvas as *mut _,
			x: self.x,
			y: self.y,
			phantom: std::marker::PhantomData,
		};

		self.advance();

		return Some(value);
	}
}*/