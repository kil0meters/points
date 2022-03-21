use bevy::{prelude::*, utils::HashSet};
// use bevy_prototype_debug_lines::DebugLines;
use std::cmp::Eq;
use std::hash::Hash;

#[derive(Debug)]
struct QuadTreeChildren<T: Copy + Hash + Eq> {
    northeast: QuadTree<T>,
    northwest: QuadTree<T>,
    southeast: QuadTree<T>,
    southwest: QuadTree<T>,
}

impl<T: Copy + Hash + Eq> QuadTreeChildren<T> {
    fn insert(&mut self, position: Vec2, item: T) -> bool {
        self.northeast.insert(position, item)
            || self.northwest.insert(position, item)
            || self.southeast.insert(position, item)
            || self.southwest.insert(position, item)
    }
}

#[derive(Debug)]
pub struct QuadTree<T: Copy + Hash + Eq> {
    x: f32,
    y: f32,
    width: f32,
    height: f32,

    max_points: usize,

    points: Option<Vec<(Vec2, T)>>,
    children: Option<Box<QuadTreeChildren<T>>>,
}

impl<T: Copy + Hash + Eq> QuadTree<T> {
    pub fn new(x: f32, y: f32, width: f32, height: f32, max_points: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
            max_points,
            points: Some(Vec::with_capacity(max_points)),
            children: None,
        }
    }

    fn subdivide(&mut self) {
        let width = self.width / 2.0;
        let height = self.height / 2.0;

        self.children = Some(Box::new(QuadTreeChildren {
            northwest: QuadTree::new(self.x, self.y, width, height, self.max_points),
            northeast: QuadTree::new(self.x + width, self.y, width, height, self.max_points),
            southwest: QuadTree::new(self.x, self.y + height, width, height, self.max_points),
            southeast: QuadTree::new(
                self.x + width,
                self.y + height,
                width,
                height,
                self.max_points,
            ),
        }));

        // We just set children so it cannot be None
        let children = self.children.as_mut().unwrap();

        // subdivide is only called when points is not None
        // TODO: We should look to avoid copying this
        let points = self.points.clone().unwrap();

        for (position, item) in points {
            children.insert(position, item);
        }

        self.points = None;
    }

    pub fn insert(&mut self, position: Vec2, item: T) -> bool {
        if position.x >= self.x
            && position.x <= self.x + self.width
            && position.y >= self.y
            && position.y <= self.y + self.height
        {
            if let Some(points) = self.points.as_mut() {
                if points.len() == self.max_points {
                    // If we've reached the maximum points, we simply subdivide and try again.
                    self.subdivide();
                    self.insert(position, item);
                } else {
                    points.push((position, item));
                }
            } else {
                let children = self.children.as_mut().unwrap();
                children.insert(position, item);
            }
            true
        } else {
            false
        }
    }

    // pub fn draw(&self, lines: &mut DebugLines) {
    //     lines.line(
    //         Vec3::new(self.x, self.y, 0.0),
    //         Vec3::new(self.x + self.width, self.y, 0.0),
    //         0.2,
    //     );
    //     lines.line(
    //         Vec3::new(self.x, self.y, 0.0),
    //         Vec3::new(self.x, self.y + self.height, 0.0),
    //         0.2,
    //     );
    //     lines.line(
    //         Vec3::new(self.x + self.width, self.y, 0.0),
    //         Vec3::new(self.x + self.width, self.y + self.height, 0.0),
    //         0.2,
    //     );
    //     lines.line(
    //         Vec3::new(self.x, self.y + self.height, 0.0),
    //         Vec3::new(self.x + self.width, self.y + self.height, 0.0),
    //         0.2,
    //     );

    //     if let Some(children) = self.children.as_ref() {
    //         children.northwest.draw(lines);
    //         children.northeast.draw(lines);
    //         children.southwest.draw(lines);
    //         children.southeast.draw(lines);
    //     }
    // }

    pub fn query(&self, shape: &impl Queryable, points: &mut HashSet<T>) {
        if shape.intersects_rectangle(self.x, self.y, self.width, self.height) {
            if let Some(children) = self.children.as_ref() {
                children.northwest.query(shape, points);
                children.northeast.query(shape, points);
                children.southwest.query(shape, points);
                children.southeast.query(shape, points);
            } else {
                // we know points is Some because children is None
                for (point, item) in self.points.as_ref().unwrap() {
                    if shape.intersects_point(point.x, point.y) {
                        points.insert(*item);
                    }
                }
            }
        }
    }
}

pub trait Queryable {
    fn intersects_rectangle(&self, x: f32, y: f32, width: f32, height: f32) -> bool;
    fn intersects_point(&self, x: f32, y: f32) -> bool;
}

pub struct Circle {
    x: f32,
    y: f32,
    r: f32,
}

impl Circle {
    pub fn new(x: f32, y: f32, r: f32) -> Self {
        Self { x, y, r }
    }
}

impl Queryable for Circle {
    fn intersects_rectangle(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        // find nearest x and y
        let cx = self.x.max(x).min(x + width);
        let cy = self.y.max(y).min(y + height);

        let dx = self.x - cx;
        let dy = self.y - cy;

        dy * dy + dx * dx <= self.r * self.r
    }

    fn intersects_point(&self, x: f32, y: f32) -> bool {
        let dx = self.x - x;
        let dy = self.y - y;

        dy * dy + dx * dx <= self.r * self.r
    }
}
