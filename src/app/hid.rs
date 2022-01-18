use super::*;

impl<'a> App<'a> {
    pub fn hid(&mut self, delta_time: f32) -> i32 {
        MOUSE.with(|m| {
            if self.camera.1 == XMVector::set(0.0, 0.0, 1.0, 0.0) {
                m.set(Position {
                    x: unsafe { GetSystemMetrics(0) / 2 },
                    y: unsafe { GetSystemMetrics(1) / 2 },
                })
            }
            let p = m.get();
            let x_t = unsafe { GetSystemMetrics(0) };
            let y_t = unsafe { GetSystemMetrics(1) };
            let (x, y) = (p.x, p.y);
            let x_angle = (x as f32 / x_t as f32 - 0.5) * PI;
            let y_angle = ((y as f32 / y_t as f32 - 0.5) * TAU)
                .clamp(-FRAC_PI_2 + f32::EPSILON, FRAC_PI_2 - f32::EPSILON);
            let x_rot = XMMatrix(XMMatrixRotationX(y_angle));

            let y_rot = XMMatrix(XMMatrixRotationY(x_angle));
            let rot = x_rot * y_rot;
            self.camera.1 = XMVector(XMVector3Transform(XMVectorSet(0.0, 0.0, 1.0, 1.0), rot.0));
            KEYS.with(|k| {
                let x = k.borrow();
                if utils::read_key(VK_ESCAPE as u16, *x) {
                    unsafe { DestroyWindow(self.h_wnd) };
                    return -1;
                }
                if utils::read_key('W' as u16, *x) {
                    self.camera.0 += XMVector(XMVector3Transform(
                        XMVectorSet(0.0, 0.0, delta_time * 3., 0.0),
                        y_rot.0,
                    ));
                }
                if utils::read_key('S' as u16, *x) {
                    self.camera.0 += XMVector(XMVector3Transform(
                        XMVectorSet(0.0, 0.0, -delta_time * 3., 0.0),
                        y_rot.0,
                    ));
                }
                if utils::read_key('A' as u16, *x) {
                    self.camera.0 += XMVector(XMVector3Transform(
                        XMVectorSet(-delta_time * 3., 0.0, 0.0, 0.0),
                        y_rot.0,
                    ));
                }
                if utils::read_key('D' as u16, *x) {
                    self.camera.0 += XMVector(XMVector3Transform(
                        XMVectorSet(delta_time * 3., 0.0, 0.0, 0.0),
                        y_rot.0,
                    ));
                }
                if utils::read_key('Q' as u16, *x) {
                    self.camera.0 += XMVector(XMVectorSet(0.0, -delta_time * 3., 0.0, 0.0));
                }
                if utils::read_key('E' as u16, *x) {
                    self.camera.0 += XMVector(XMVectorSet(0.0, delta_time * 3., 0.0, 0.0));
                }
                if utils::read_key('L' as u16, *x) {
                    self.cubes.pop();
                    let (verticies, indicies) = self.cubes.to_vertices(384, 312);
                    let (verticies, indicies) = if verticies.is_empty() {
                        self.cubes = cubes::Cubes::new_list(vec![
                            Coord { x: -3, y: 0, z: 0 },
                            Coord { x: -2, y: 0, z: 0 },
                            Coord { x: -2, y: 1, z: 1 },
                            Coord { x: -1, y: 0, z: 0 },
                            Coord { x: -1, y: 0, z: 1 },
                            Coord { x: -1, y: 1, z: 1 },
                        ])
                        .unwrap();
                        self.cubes.to_vertices(384, 312)
                    } else {
                        (verticies, indicies)
                    };
                    unsafe {
                        self.d_device_context.UpdateSubresource(
                            <*mut _>::cast(self.d_vertex_buffer),
                            0,
                            ptr::null(),
                            <*const _>::cast(verticies.as_ptr()),
                            0,
                            0,
                        )
                    };
                    unsafe {
                        self.d_device_context.UpdateSubresource(
                            <*mut _>::cast(self.d_index_buffer),
                            0,
                            ptr::null(),
                            <*const _>::cast(indicies.as_ptr()),
                            0,
                            0,
                        )
                    };
                    return indicies.len() as i32;
                }
                0
            })
        })
    }
}
