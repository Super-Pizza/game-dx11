use super::*;
#[derive(Debug, PartialEq, Eq)]
pub struct Cubes {
    positions: Vec<u8>,
    start_x: i32,
    start_y: i32,
    start_z: i32,
    len_x: usize,
    len_y: usize,
    len_z: usize,
}
impl Cubes {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            start_x: 0,
            start_y: 0,
            start_z: 0,
            len_x: 0,
            len_y: 0,
            len_z: 0,
        }
    }
    pub fn new_list(list: Vec<Coord<i32>>) -> Option<Self> {
        let mut positions = vec![0];
        let list_iter = &list;
        let Coord { x: start_x, .. } = list_iter.iter().min_by(|x, y| x.x.cmp(&y.x))?;
        let Coord { y: start_y, .. } = list_iter.iter().min_by(|x, y| x.y.cmp(&y.y))?;
        let Coord { z: start_z, .. } = list_iter.iter().min_by(|x, y| x.z.cmp(&y.z))?;
        let Coord { x: len_x, .. } = list_iter.iter().max_by(|x, y| x.x.cmp(&y.x))?;
        let Coord { y: len_y, .. } = list_iter.iter().max_by(|x, y| x.y.cmp(&y.y))?;
        let Coord { z: len_z, .. } = list_iter.iter().max_by(|x, y| x.z.cmp(&y.z))?;
        let (start_x, start_y, start_z) = (*start_x, *start_y, *start_z);
        let (len_x, len_y, len_z) = (
            *len_x - start_x + 1,
            *len_y - start_y + 1,
            *len_z - start_z + 1,
        );
        let stride_z = len_x;
        let stride_y = stride_z * len_z;

        let len_vec = stride_y * len_y;
        positions.resize(len_vec as usize, 0);
        for item in list_iter {
            let pos_x = item.x - start_x;
            let pos_y = (item.y - start_y) * stride_y;
            let pos_z = (item.z - start_z) * stride_z;
            let pos = pos_x + pos_y + pos_z;
            positions[pos as usize] = 1;
        }
        Some(Self {
            positions,
            start_x,
            start_y,
            start_z,
            len_x: len_x as usize,
            len_y: len_y as usize,
            len_z: len_z as usize,
        })
    }
    pub fn to_vertices(&self, vert_len: usize, idx_len: usize) -> (Vec<Coord<f32>>, Vec<u16>) {
        if self.positions.is_empty() {
            return (vec![], vec![]);
        }
        #[rustfmt::skip]
        const VERTICIES: [Coord<f32>; 8] = [
            Coord { x: 0., y: 0., z: 0. },
            Coord { x: 0., y: 0., z: 1. },
            Coord { x: 0., y: 1., z: 1. },
            Coord { x: 0., y: 1., z: 0. },
            Coord { x: 1., y: 1., z: 0. },
            Coord { x: 1., y: 1., z: 1. },
            Coord { x: 1., y: 0., z: 1. },
            Coord { x: 1., y: 0., z: 0. },
        ];
        const INDICIES: [u16; 36] = [
            0, 1, 2, 0, 2, 3, //Left
            4, 5, 6, 4, 6, 7, //Right
            0, 3, 4, 0, 4, 7, //Front
            1, 6, 5, 1, 5, 2, //Back
            2, 4, 3, 2, 5, 4, //Up
            0, 6, 1, 0, 7, 6, //Down
        ];
        let stride_z = self.len_x;
        let stride_y = stride_z * self.len_z;
        let len_vec = stride_y * self.len_y;
        let mut indicies_all = Vec::with_capacity(idx_len);
        let mut shift = 0;
        let mut verticies_all = Vec::with_capacity(vert_len);
        for cube in self.positions.iter().enumerate() {
            if *cube.1 == 1 {
                if cube.0 % stride_z == 0 || self.positions[cube.0 - 1] == 0 {
                    indicies_all.extend(INDICIES[0..6].iter().map(|i| i + shift));
                }
                if cube.0 % stride_z == stride_z - 1 || self.positions[cube.0 + 1] == 0 {
                    indicies_all.extend(INDICIES[6..12].iter().map(|i| i + shift));
                }
                if cube.0 % stride_y < stride_z || self.positions[cube.0 - stride_z] == 0 {
                    indicies_all.extend(INDICIES[12..18].iter().map(|i| i + shift));
                }
                if cube.0 % stride_y >= stride_y - stride_z
                    || self.positions[cube.0 + stride_z] == 0
                {
                    indicies_all.extend(INDICIES[18..24].iter().map(|i| i + shift));
                }
                if cube.0 >= len_vec - stride_y || self.positions[cube.0 + stride_y] == 0 {
                    indicies_all.extend(INDICIES[24..30].iter().map(|i| i + shift));
                }
                if cube.0 < stride_y || self.positions[cube.0 - stride_y] == 0 {
                    indicies_all.extend(INDICIES[30..36].iter().map(|i| i + shift));
                }
                shift += 8;
                let x = ((cube.0 % stride_z) as i32 + self.start_x) as f32;
                let z = (((cube.0 % stride_y) / stride_z) as i32 + self.start_y) as f32;
                let y = ((cube.0 / stride_y) as i32 + self.start_z) as f32;
                let coord = Coord { x, y, z };
                verticies_all.extend(VERTICIES.iter().map(|v| *v + coord));
            }
        }
        (verticies_all, indicies_all)
    }
    pub fn pop(&mut self) -> Option<()> {
        let idx = self.positions.iter().rposition(|&x| x == 1)?;
        self.positions[idx as usize] = 0;
        //self.shrink();
        Some(())
    }
    /*pub fn pop_start(&mut self) -> Option<()> {
        let idx = self.positions.iter().position(|&x| x == 1)?;
        self.positions[idx as usize] = 0;
        //self.shrink();
        Some(())
    }*/
    pub fn remove(&mut self, x: usize, y: usize, z: usize) -> bool {
        let pos = x + self.len_x * z + self.len_x * self.len_z * y;
        let state = self.positions[pos];
        self.positions[pos] = 0;
        state > 0
    } /*
      pub fn shrink(&mut self) {
          if self.positions.iter().all(|&x| x == 0) {
              self.positions = vec![];
              self.len_x = 0;
              self.len_y = 0;
              self.len_z = 0;
              return;
          }
          let mut left = 0;
          let mut right = 0;
          let mut down = self.len_y;
          let mut up = 0;
          let mut front = self.len_z;
          let mut back = 0;
          let mut stop_left = false;
          for i in 0..self.len_x {
              let mut stop_front = false;
              let mut cube_x = false;
              let mut this_z = 0;
              for j in 0..self.len_z {
                  let mut cube_z = false;
                  for k in 0..self.len_y {
                      let idx = (j + k * self.len_z) * self.len_x + i;
                      if self.positions[idx] != 0 {
                          cube_x = true;
                          cube_z = true;
                          up = up.max(k + 1);
                          down = down.min(k);
                      }
                  }
                  if cube_z {
                      back = back.max(j + 1);
                      stop_front = true;
                  } else if !stop_front {
                      this_z += 1;
                  }
              }
              if cube_x {
                  right = right.max(i + 1);
                  stop_left = true;
              } else if !stop_left {
                  left += 1;
              }
              front = front.min(this_z);
          }
          let mut resized = vec![];
          let stride_y = self.len_x * self.len_z;
          println!("{}-{},{}-{},{}-{}", left, right, front, back, down, up);
          for item in self.positions.iter().enumerate() {
              if item.0 % self.len_x >= left
                  && item.0 % self.len_x < right
                  && item.0 % stride_y >= front * self.len_x
                  && item.0 % stride_y < back * self.len_x
                  && item.0 >= down * stride_y
                  && item.0 < up * stride_y
              {
                  resized.push(*item.1);
              }
          }
          self.positions = resized;
          self.len_x = right - left;
          self.len_y = up - down;
          self.len_z = back - front;
          self.start_x += left as i32;
          self.start_y += down as i32;
          self.start_z += front as i32;
      }*/
}

#[test]
#[rustfmt::skip::macros(assert_eq)]
fn cube_new_list() {
    assert_eq!(Cubes::new_list(vec![
        Coord {x: -5, y: -3, z: -2},
        Coord {x: -2, y: -3, z: 2},
        Coord {x: -3, y: -1, z: 0},
        Coord {x: -4, y: -2, z: -1},
        Coord {x: 0, y: 0, z: -2},
        Coord {x: -1, y: -1, z: 1},
    ]).unwrap(),
    Cubes {
        positions: vec![
            1,0,0,0,0,0,//z:-2,y:-3
            0,0,0,0,0,0,//z:-1,y:-3
            0,0,0,0,0,0,//z:0,y:-3
            0,0,0,0,0,0,//z:1,y:-3
            0,0,0,1,0,0,//z:2,y:-3
            0,0,0,0,0,0,//z:-2,y:-2
            0,1,0,0,0,0,//z:-1,y:-2
            0,0,0,0,0,0,//z:0,y:-2
            0,0,0,0,0,0,//z:1,y:-2
            0,0,0,0,0,0,//z:2,y:-2
            0,0,0,0,0,0,//z:-2,y:-1
            0,0,0,0,0,0,//z:-1,y:-1
            0,0,1,0,0,0,//z:0,y:-1
            0,0,0,0,1,0,//z:1,y:-1
            0,0,0,0,0,0,//z:2,y:-1
            0,0,0,0,0,1,//z:-2,y:0
            0,0,0,0,0,0,//z:-1,y:0
            0,0,0,0,0,0,//z:0,y:0
            0,0,0,0,0,0,//z:1,y:0
            0,0,0,0,0,0,//z:2,y:0
        ],
        start_x: -5,
        len_x: 6,
        start_y: -3,
        len_y: 4,
        start_z: -2,
        len_z: 5,
    })
}
#[test]
#[rustfmt::skip::macros(assert_eq)]
fn cube_to_vertices() {
    let cubes = Cubes::new_list(vec![
        Coord { x: -3, y: 0, z: 0 },
        Coord { x: -2, y: 0, z: 0 },
        Coord { x: -2, y: 1, z: 0 },
        Coord { x: -1, y: 0, z: 0 },
        Coord { x: -1, y: 0, z: 1 },
        Coord { x: -1, y: 1, z: 1 },
    ])
    .unwrap();
    assert_eq!(
        cubes.to_vertices(384, 312),
        (vec![
            Coord { x: 0., y: 0., z: 0. },//1.
            Coord { x: 0., y: 0., z: 1. },//1.
            Coord { x: 0., y: 1., z: 1. },//1.
            Coord { x: 0., y: 1., z: 0. },//1.
            Coord { x: 1., y: 1., z: 0. },//1.
            Coord { x: 1., y: 1., z: 1. },//1.
            Coord { x: 1., y: 0., z: 1. },//1.
            Coord { x: 1., y: 0., z: 0. },//1.
            Coord { x: 0.+1., y: 0., z: 0. },//2.
            Coord { x: 0.+1., y: 0., z: 1. },//2.
            Coord { x: 0.+1., y: 1., z: 1. },//2.
            Coord { x: 0.+1., y: 1., z: 0. },//2.
            Coord { x: 1.+1., y: 1., z: 0. },//2.
            Coord { x: 1.+1., y: 1., z: 1. },//2.
            Coord { x: 1.+1., y: 0., z: 1. },//2.
            Coord { x: 1.+1., y: 0., z: 0. },//2.
            Coord { x: 0.+2., y: 0., z: 0. },//4.
            Coord { x: 0.+2., y: 0., z: 1. },//4.
            Coord { x: 0.+2., y: 1., z: 1. },//4.
            Coord { x: 0.+2., y: 1., z: 0. },//4.
            Coord { x: 1.+2., y: 1., z: 0. },//4.
            Coord { x: 1.+2., y: 1., z: 1. },//4.
            Coord { x: 1.+2., y: 0., z: 1. },//4.
            Coord { x: 1.+2., y: 0., z: 0. },//4.
            Coord { x: 0.+2., y: 0., z: 0.+1. },//5.
            Coord { x: 0.+2., y: 0., z: 1.+1. },//5.
            Coord { x: 0.+2., y: 1., z: 1.+1. },//5.
            Coord { x: 0.+2., y: 1., z: 0.+1. },//5.
            Coord { x: 1.+2., y: 1., z: 0.+1. },//5.
            Coord { x: 1.+2., y: 1., z: 1.+1. },//5.
            Coord { x: 1.+2., y: 0., z: 1.+1. },//5.
            Coord { x: 1.+2., y: 0., z: 0.+1. },//5.
            Coord { x: 0.+1., y: 0.+1., z: 0. },//3.
            Coord { x: 0.+1., y: 0.+1., z: 1. },//3.
            Coord { x: 0.+1., y: 1.+1., z: 1. },//3.
            Coord { x: 0.+1., y: 1.+1., z: 0. },//3.
            Coord { x: 1.+1., y: 1.+1., z: 0. },//3.
            Coord { x: 1.+1., y: 1.+1., z: 1. },//3.
            Coord { x: 1.+1., y: 0.+1., z: 1. },//3.
            Coord { x: 1.+1., y: 0.+1., z: 0. },//3.
            Coord { x: 0.+2., y: 0.+1., z: 0.+1. },//6.
            Coord { x: 0.+2., y: 0.+1., z: 1.+1. },//6.
            Coord { x: 0.+2., y: 1.+1., z: 1.+1. },//6.
            Coord { x: 0.+2., y: 1.+1., z: 0.+1. },//6.
            Coord { x: 1.+2., y: 1.+1., z: 0.+1. },//6.
            Coord { x: 1.+2., y: 1.+1., z: 1.+1. },//6.
            Coord { x: 1.+2., y: 0.+1., z: 1.+1. },//6.
            Coord { x: 1.+2., y: 0.+1., z: 0.+1. },//6.
        ],vec![
            0, 1, 2, 0, 2, 3, //1
            //4, 5, 6, 4, 6, 7, //1
            0, 3, 4, 0, 4, 7, //1
            1, 6, 5, 1, 5, 2, //1
            2, 4, 3, 2, 5, 4, //1
            0, 6, 1, 0, 7, 6, //1
            //0+8, 1+8, 2+8, 0+8, 2+8, 3+8, //2
            //4+8, 5+8, 6+8, 4+8, 6+8, 7+8, //2
            0+8, 3+8, 4+8, 0+8, 4+8, 7+8, //2
            1+8, 6+8, 5+8, 1+8, 5+8, 2+8, //2
            //2+8, 4+8, 3+8, 2+8, 5+8, 4+8, //2
            0+8, 6+8, 1+8, 0+8, 7+8, 6+8, //2
            //0+16, 1+16, 2+16, 0+16, 2+16, 3+16, //3
            4+16, 5+16, 6+16, 4+16, 6+16, 7+16, //3
            0+16, 3+16, 4+16, 0+16, 4+16, 7+16, //3
            //1+16, 6+16, 5+16, 1+16, 5+16, 2+16, //3
            2+16, 4+16, 3+16, 2+16, 5+16, 4+16, //3
            0+16, 6+16, 1+16, 0+16, 7+16, 6+16, //3
            0+24, 1+24, 2+24, 0+24, 2+24, 3+24, //4
            4+24, 5+24, 6+24, 4+24, 6+24, 7+24, //4
            //0+24, 3+24, 4+24, 0+24, 4+24, 7+24, //4
            1+24, 6+24, 5+24, 1+24, 5+24, 2+24, //4
            //2+24, 4+24, 3+24, 2+24, 5+24, 4+24, //4
            0+24, 6+24, 1+24, 0+24, 7+24, 6+24, //4
            0+32, 1+32, 2+32, 0+32, 2+32, 3+32, //5
            4+32, 5+32, 6+32, 4+32, 6+32, 7+32, //5
            0+32, 3+32, 4+32, 0+32, 4+32, 7+32, //5
            1+32, 6+32, 5+32, 1+32, 5+32, 2+32, //5
            2+32, 4+32, 3+32, 2+32, 5+32, 4+32, //5
            //0+32, 6+32, 1+32, 0+32, 7+32, 6+32, //5
            0+40, 1+40, 2+40, 0+40, 2+40, 3+40, //6
            4+40, 5+40, 6+40, 4+40, 6+40, 7+40, //6
            0+40, 3+40, 4+40, 0+40, 4+40, 7+40, //6
            1+40, 6+40, 5+40, 1+40, 5+40, 2+40, //6
            2+40, 4+40, 3+40, 2+40, 5+40, 4+40, //6
            //0+40, 6+40, 1+40, 0+40, 7+40, 6+40, //6
        ])
    );
}

#[test]
#[rustfmt::skip::macros(assert)]
fn cube_remove() {
    assert!(Cubes::new_list(vec![
        Coord {x: -5, y: -3, z: -2},
        Coord {x: -2, y: -3, z: 2},
        Coord {x: -3, y: -1, z: 0},
        Coord {x: -4, y: -2, z: -1},
        Coord {x: 0, y: 0, z: -2},
        Coord {x: -1, y: -1, z: 1},
    ]).unwrap().remove(2, 2, 2))
}

impl Default for Cubes {
    fn default() -> Self {
        Self::new()
    }
}
