use super::*;

impl<'a> App<'a> {
    pub fn init_buffers<'b>(
        device: &ID3D11Device,
        verticies: &Vec<Coord<f32>>,
        indicies: &Vec<u16>,
    ) -> HResult<[&'b mut ID3D11Buffer; 2]> {
        let mut vertex_buffer: *mut ID3D11Buffer = ptr::null_mut();
        let vertex_buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: (size_of::<Coord<f32>>() * verticies.len() * 2) as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let resource_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: <*const _>::cast(verticies.as_ptr()),
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };
        let result =
            unsafe { device.CreateBuffer(&vertex_buffer_desc, &resource_data, &mut vertex_buffer) };
        if result != 0 {
            dbg!();
            return Err(result);
        }
        let mut index_buffer: *mut ID3D11Buffer = ptr::null_mut();
        let index_buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: (size_of::<u16>() * indicies.len() * 2) as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_INDEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let resource_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: <*const _>::cast(indicies.as_ptr()),
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };
        let result =
            unsafe { device.CreateBuffer(&index_buffer_desc, &resource_data, &mut index_buffer) };
        if result != 0 {
            dbg!();
            return Err(result);
        }
        Ok(unsafe { [&mut *vertex_buffer, &mut *index_buffer] })
    }
    pub fn init_const_buffers<'b>(device: &ID3D11Device) -> HResult<[&'b mut ID3D11Buffer; 3]> {
        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: size_of::<XMMatrix>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let mut const_buffers: [*mut ID3D11Buffer; 3] = [ptr::null_mut(); 3];
        let result =
            unsafe { device.CreateBuffer(&buffer_desc, ptr::null(), &mut const_buffers[0]) };
        if result != 0 {
            dbg!();
            return Err(result);
        }
        let result =
            unsafe { device.CreateBuffer(&buffer_desc, ptr::null(), &mut const_buffers[1]) };
        if result != 0 {
            dbg!();
            return Err(result);
        }
        let result =
            unsafe { device.CreateBuffer(&buffer_desc, ptr::null(), &mut const_buffers[2]) };
        if result != 0 {
            dbg!();
            return Err(result);
        }
        Ok(const_buffers.map(|e| unsafe { &mut *e }))
    }
    pub fn load_shaders<'b>(
        device: &ID3D11Device,
    ) -> HResult<(&'b mut ID3D11VertexShader, &'b mut ID3D11PixelShader)> {
        let mut vertex_shader: *mut ID3D11VertexShader = ptr::null_mut();
        let mut pixel_shader: *mut ID3D11PixelShader = ptr::null_mut();
        let result = unsafe {
            device.CreateVertexShader(
                <*const _>::cast(VERTEX_SHADER_DATA.as_ptr()),
                VERTEX_SHADER_DATA.len(),
                ptr::null_mut(),
                &mut vertex_shader,
            )
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        let result = unsafe {
            device.CreatePixelShader(
                <*const _>::cast(PIXEL_SHADER_DATA.as_ptr()),
                PIXEL_SHADER_DATA.len(),
                ptr::null_mut(),
                &mut pixel_shader,
            )
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        Ok(unsafe { (&mut *vertex_shader, &mut *pixel_shader) })
    }
    pub fn init_input_layout<'b>(device: &ID3D11Device) -> HResult<&'b mut ID3D11InputLayout> {
        let vertex_input_desc = [D3D11_INPUT_ELEMENT_DESC {
            SemanticName: <*const _>::cast("POSITION\0".as_ptr()),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 0,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        }];
        let mut input_layout: *mut ID3D11InputLayout = ptr::null_mut();
        let result = unsafe {
            device.CreateInputLayout(
                vertex_input_desc.as_ptr(),
                1,
                <*const _>::cast(VERTEX_SHADER_DATA.as_ptr()),
                VERTEX_SHADER_DATA.len(),
                &mut input_layout,
            )
        };
        if result != 0 {
            return Err(result);
        }
        Ok(unsafe { &mut *input_layout })
    }
    pub fn init_matrices(
        width: i32,
        height: i32,
        device_context: &ID3D11DeviceContext,
        constant_buffers: &mut [&mut ID3D11Buffer; 3],
    ) -> [XMMatrix; 3] {
        let matrix = XMMatrixPerspectiveFovLH(
            XMConvertToRadians(45.0),
            width as f32 / height as f32,
            0.1,
            100.0,
        );
        unsafe {
            device_context.UpdateSubresource(
                <*mut _>::cast(constant_buffers[CB_APP] as *mut _),
                0,
                ptr::null(),
                <*const _>::cast(&matrix as *const _),
                0,
                0,
            )
        };
        let eye_position = XMVector::set(0.0, 0.0, -10.0, 1.0);
        let focus_point = XMVector::set(0.0, 0.0, 0.0, 1.0);
        let up_direction = XMVector::set(0.0, 1.0, 0.0, 0.0);
        let view_matrix = XMMatrixLookAtLH(eye_position.0, focus_point.0, up_direction.0);
        unsafe {
            device_context.UpdateSubresource(
                <*mut _>::cast(constant_buffers[CB_FRAME] as *mut _),
                0,
                ptr::null(),
                <*const _>::cast(&view_matrix as *const _),
                0,
                0,
            )
        };
        let rotation_axis = XMVector::set(0.0, 1.0, 1.0, 0.0);
        let world_matrix = XMMatrixRotationAxis(rotation_axis.0, 0.0);
        unsafe {
            device_context.UpdateSubresource(
                <*mut _>::cast(constant_buffers[CB_OBJECT] as *mut _),
                0,
                ptr::null(),
                <*const _>::cast(&world_matrix as *const _),
                0,
                0,
            )
        };
        [
            XMMatrix(matrix),
            XMMatrix(view_matrix),
            XMMatrix(world_matrix),
        ]
    }
    pub fn load_assets() -> HResult<Vec<Asset>> {
        let file = OpenOptions::new()
            .read(true)
            .open(Path::new(
                "C:\\Users\\fabie\\Documents\\Graphics\\game\\UI.png",
            ))
            .map_err(io_error)?;
        let mut data = Decoder::new(file).read_info().map_err(|e| match e {
            DecodingError::IoError(x) => io_error(x),
            DecodingError::Format(x) => {
                eprintln!("{}", x);
                -2147024883
            }
            DecodingError::Parameter(_) => unreachable!(),
            DecodingError::LimitsExceeded => -2147016669,
        })?;
        let info = data.info();
        let mut buf = Vec::with_capacity(info.raw_bytes());
        let info = data.next_frame(&mut buf).map_err(|e| match e {
            DecodingError::IoError(x) => io_error(x),
            DecodingError::Format(x) => {
                eprintln!("{}", x);
                -2147024883
            }
            DecodingError::Parameter(_) => unreachable!(),
            DecodingError::LimitsExceeded => -2147016669,
        })?;
        let asset = Asset { buf, info };
        let assets = vec![asset];
        Ok(assets)
    }
}
