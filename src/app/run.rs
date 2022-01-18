use super::*;
impl<'a> App<'a> {
    pub fn update(&mut self) {
        let eye_position = self.camera.0;
        let eye_direction = self.camera.1;
        let up_direction = XMVector::set(0.0, 1.0, 0.0, 0.0);
        let view_matrix = XMMatrixLookToLH(eye_position.0, eye_direction.0, up_direction.0);
        unsafe {
            self.d_device_context.UpdateSubresource(
                <*mut _>::cast(self.d_constant_buffers[CB_FRAME] as *mut _),
                0,
                ptr::null(),
                <*const _>::cast(&view_matrix as *const _),
                0,
                0,
            )
        };
        self.view_matrix = XMMatrix(view_matrix);
        let rotation_axis = XMVector::set(0.0, 1.0, 1.0, 0.0);
        let world_matrix = XMMatrixRotationAxis(rotation_axis.0, 0.0);
        unsafe {
            self.d_device_context.UpdateSubresource(
                <*mut _>::cast(self.d_constant_buffers[CB_OBJECT] as *mut _),
                0,
                ptr::null(),
                <*const _>::cast(&world_matrix as *const _),
                0,
                0,
            )
        };
        self.world_matrix = XMMatrix(world_matrix);
    }
    pub fn clear(&mut self, color: [f32; 4], clear_depth: f32, clear_stencil: u8) {
        unsafe {
            self.d_device_context
                .ClearRenderTargetView(self.d_render_target_view as *mut _, &color);
            self.d_device_context.ClearDepthStencilView(
                self.d_depth_stencil_view as *mut _,
                D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL,
                clear_depth,
                clear_stencil,
            );
        }
    }
    pub fn present(&mut self) {
        if self.flags.vsync {
            unsafe {
                self.d_swapchain.Present(1, 0);
            }
        } else {
            unsafe {
                self.d_swapchain.Present(0, 0);
            }
        }
    }
    pub fn render(&mut self, indicies: i32) {
        assert!(!(self.d_device_context as *mut ID3D11DeviceContext).is_null());
        assert!(!(self.d_device as *mut ID3D11Device).is_null());
        self.clear([0.3921569, 0.58431375, 0.9294119, 1.0], 1.0, 0);
        let vertex_stride = size_of::<utils::Coord<f32>>();
        let offset = 0;
        let indicies = match self.state {
            State::InGame(_) => indicies,
            _ => 0,
        };
        unsafe {
            self.d_device_context.IASetVertexBuffers(
                0,
                1,
                &(self.d_vertex_buffer as *mut _) as *const *mut _,
                &(vertex_stride as u32),
                &offset,
            );
            self.d_device_context
                .IASetInputLayout(self.d_input_layout as *mut _);
            self.d_device_context
                .IASetIndexBuffer(self.d_index_buffer, DXGI_FORMAT_R16_UINT, 0);
            self.d_device_context
                .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.d_device_context
                .VSSetShader(self.d_vertex_shader as *mut _, ptr::null(), 0);
            self.d_device_context.VSSetConstantBuffers(
                0,
                3,
                self.d_constant_buffers.as_ptr() as *const *mut _,
            );
            self.d_device_context.RSSetState(self.d_rasterizer_state);
            self.d_device_context.RSSetViewports(1, &self.d_viewport);
            self.d_device_context
                .PSSetShader(self.d_pixel_shader, ptr::null(), 0);
            self.d_device_context.OMSetRenderTargets(
                1,
                &(self.d_render_target_view as *mut _) as *const *mut _,
                self.d_depth_stencil_view,
            );
            self.d_device_context
                .OMSetDepthStencilState(self.d_depth_stencil_state, 1);
            self.d_device_context.DrawIndexed((indicies) as u32, 0, 0);
            self.present();
        }
    }
}
