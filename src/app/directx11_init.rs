use super::*;

impl<'a> App<'a> {
    pub fn init_application(flags: self::Flags, class_name: &str) -> HResult<HWND> {
        let h_instance = unsafe { GetModuleHandleW(ptr::null()) };
        let win_name = String::from("A Game");
        let window = utils::str_to_c16(&win_name);
        let class = utils::str_to_c16(class_name);
        let (win_width, win_height) = match flags.state {
            WindowState::Windowed(x, y) => (x as i32, y as i32),
            WindowState::Maximized => unsafe { (GetSystemMetrics(61), GetSystemMetrics(62)) },
            WindowState::Fullscreen => unsafe { (GetSystemMetrics(0), GetSystemMetrics(1)) },
        };
        let wnd_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: unsafe { LoadIconW(ptr::null_mut(), IDI_APPLICATION) },
            hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: ptr::null(),
            lpszClassName: class.as_ptr(),
            hIconSm: unsafe { LoadIconW(ptr::null_mut(), IDI_APPLICATION) },
        };
        if unsafe { RegisterClassExW(&wnd_class) } == 0 {
            dbg!();
            return Err(unsafe { utils::win32_to_hresult(GetLastError()) });
        };
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: win_width,
            bottom: win_height,
        };
        unsafe { AdjustWindowRectEx(&mut rect, 0, FALSE, 0) };
        let a_width = rect.right - rect.left;
        let a_height = rect.bottom - rect.top;
        let h_wnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                window.as_ptr(),
                match flags.state {
                    WindowState::Maximized => WS_OVERLAPPEDWINDOW | WS_MAXIMIZE,
                    WindowState::Fullscreen => WS_POPUP,
                    _ => WS_OVERLAPPEDWINDOW,
                },
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                a_width,
                a_height,
                ptr::null_mut(),
                ptr::null_mut(),
                h_instance,
                ptr::null_mut(),
            )
        };
        if h_wnd.is_null() {
            dbg!();
            return Err(unsafe { utils::win32_to_hresult(GetLastError()) });
        }
        unsafe { ShowWindow(h_wnd, SW_SHOW) };
        Ok(h_wnd)
    }

    pub fn init_device_and_swapchain<'b>(
        window_handle: HWND,
        flags: Flags,
    ) -> HResult<(
        &'b mut ID3D11Device,
        &'b mut ID3D11DeviceContext,
        &'b mut IDXGISwapChain,
        i32,
        i32,
    )> {
        assert!(!window_handle.is_null());
        let mut rect = RECT {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        };
        unsafe { GetClientRect(window_handle, &mut rect) };
        let client_width = rect.right - rect.left;
        let client_height = rect.bottom - rect.top;
        let swapchain_desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: client_width as u32,
                Height: client_height as u32,
                RefreshRate: DXGI_RATIONAL {
                    Numerator: 60,
                    Denominator: 1,
                },
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
            },
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            OutputWindow: window_handle,
            Windowed: if let WindowState::Fullscreen = flags.state {
                FALSE
            } else {
                TRUE
            },
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            Flags: 0,
        };
        let create_devce_flags = if cfg!(debug_assertions) {
            D3D11_CREATE_DEVICE_DEBUG
        } else {
            0
        };
        let feature_levels: [D3D_FEATURE_LEVEL; 2] =
            [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];
        let mut feature_level = D3D_FEATURE_LEVEL_11_1;
        let mut swapchain: *mut IDXGISwapChain = ptr::null_mut();
        let mut device: *mut ID3D11Device = ptr::null_mut();
        let mut immediate_context: *mut ID3D11DeviceContext = ptr::null_mut();
        let result = unsafe {
            D3D11CreateDeviceAndSwapChain(
                ptr::null_mut(),
                D3D_DRIVER_TYPE_HARDWARE,
                ptr::null_mut(),
                create_devce_flags,
                feature_levels.as_ptr(),
                feature_levels.len() as u32,
                D3D11_SDK_VERSION,
                &swapchain_desc,
                &mut swapchain,
                &mut device,
                &mut feature_level,
                &mut immediate_context,
            )
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        Ok(unsafe {
            (
                &mut *device,
                &mut *immediate_context,
                &mut *swapchain,
                client_width,
                client_height,
            )
        })
    }
    pub fn init_rtv<'b>(
        swapchain: &mut IDXGISwapChain,
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11RenderTargetView> {
        let mut back_buffer: *mut ID3D11Texture2D = ptr::null_mut();
        let result = unsafe {
            swapchain.GetBuffer(
                0,
                &ID3D11Texture2D::uuidof(),
                <*mut _>::cast(&mut back_buffer),
            )
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };

        let mut rtv: *mut ID3D11RenderTargetView = ptr::null_mut();
        let result = unsafe {
            device.CreateRenderTargetView(
                <*mut _>::cast(back_buffer as *mut _),
                ptr::null(),
                &mut rtv,
            )
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        release!(back_buffer);
        Ok(unsafe { &mut *rtv })
    }
    pub fn init_depth_stencil_buffer<'b>(
        client_width: i32,
        client_height: i32,
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11Texture2D> {
        let mut depth_stencil_buffer: *mut ID3D11Texture2D = ptr::null_mut();
        let depth_stencil_buffer_desc = D3D11_TEXTURE2D_DESC {
            ArraySize: 1,
            Width: client_width as u32,
            Height: client_height as u32,
            MipLevels: 1,
            Format: DXGI_FORMAT_D24_UNORM_S8_UINT,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_DEPTH_STENCIL,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let result = unsafe {
            device.CreateTexture2D(
                &depth_stencil_buffer_desc,
                ptr::null(),
                &mut depth_stencil_buffer,
            )
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        Ok(unsafe { &mut *depth_stencil_buffer })
    }
    pub fn init_depth_stencil_view<'b>(
        device: &mut ID3D11Device,
        depth_stencil_buffer: &mut ID3D11Texture2D,
    ) -> HResult<&'b mut ID3D11DepthStencilView> {
        let mut depth_stencil_view: *mut ID3D11DepthStencilView = ptr::null_mut();
        let result = unsafe {
            device.CreateDepthStencilView(
                <*mut _>::cast(depth_stencil_buffer as *mut _),
                ptr::null(),
                &mut depth_stencil_view,
            )
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        Ok(unsafe { &mut *depth_stencil_view })
    }
    pub fn init_depth_stencil_state<'b>(
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11DepthStencilState> {
        let mut depth_stencil_state: *mut ID3D11DepthStencilState = ptr::null_mut();
        let depth_stencil_state_desc = D3D11_DEPTH_STENCIL_DESC {
            DepthEnable: TRUE,
            DepthWriteMask: D3D11_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D11_COMPARISON_LESS,
            StencilEnable: FALSE,
            StencilReadMask: 0,
            StencilWriteMask: 0,
            FrontFace: D3D11_DEPTH_STENCILOP_DESC {
                StencilFailOp: 0,
                StencilDepthFailOp: 0,
                StencilPassOp: 0,
                StencilFunc: 0,
            },
            BackFace: D3D11_DEPTH_STENCILOP_DESC {
                StencilFailOp: 0,
                StencilDepthFailOp: 0,
                StencilPassOp: 0,
                StencilFunc: 0,
            },
        };
        let result = unsafe {
            device.CreateDepthStencilState(&depth_stencil_state_desc, &mut depth_stencil_state)
        };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        Ok(unsafe { &mut *depth_stencil_state })
    }
    pub fn init_rasterizer_state<'b>(
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11RasterizerState> {
        let mut rasterizer_state: *mut ID3D11RasterizerState = ptr::null_mut();
        let rasterizer_desc = D3D11_RASTERIZER_DESC {
            FillMode: D3D11_FILL_SOLID,
            CullMode: D3D11_CULL_BACK,
            FrontCounterClockwise: FALSE,
            DepthBias: 0,
            DepthBiasClamp: 0.0,
            SlopeScaledDepthBias: 0.0,
            DepthClipEnable: TRUE,
            ScissorEnable: FALSE,
            MultisampleEnable: FALSE,
            AntialiasedLineEnable: FALSE,
        };
        let result =
            unsafe { device.CreateRasterizerState(&rasterizer_desc, &mut rasterizer_state) };
        if result != 0 {
            dbg!();
            return Err(result);
        };
        Ok(unsafe { &mut *rasterizer_state })
    }
}
