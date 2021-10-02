//use directx_math::{XMMatrix, XMFLOAT3};
use crate::utils;
use directx_math::{
    XMConvertToRadians, XMMatrix, XMMatrixLookAtLH, XMMatrixPerspectiveFovLH, XMMatrixRotationAxis,
    XMVector, XMFLOAT3,
};
use std::{
    mem::{self, size_of, MaybeUninit},
    ptr,
    time::{Duration, Instant},
};
use winapi::{
    shared::{
        dxgi::{IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD},
        dxgiformat::{
            DXGI_FORMAT_D24_UNORM_S8_UINT, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32G32B32_FLOAT,
            DXGI_FORMAT_R8G8B8A8_UNORM,
        },
        dxgitype::{
            DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
            DXGI_RATIONAL, DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT,
        },
        minwindef::{FALSE, LPARAM, LRESULT, TRUE, UINT, WPARAM},
        ntdef::HRESULT,
        windef::{HBRUSH, HWND, RECT},
    },
    um::{
        d3d11::{
            D3D11CreateDeviceAndSwapChain, ID3D11Buffer, ID3D11DepthStencilState,
            ID3D11DepthStencilView, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
            ID3D11PixelShader, ID3D11RasterizerState, ID3D11RenderTargetView, ID3D11Texture2D,
            ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_DEPTH_STENCIL,
            D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC,
            D3D11_CLEAR_DEPTH, D3D11_CLEAR_STENCIL, D3D11_COMPARISON_LESS,
            D3D11_CREATE_DEVICE_DEBUG, D3D11_CULL_BACK, D3D11_DEPTH_STENCILOP_DESC,
            D3D11_DEPTH_STENCIL_DESC, D3D11_DEPTH_WRITE_MASK_ALL, D3D11_FILL_SOLID,
            D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_RASTERIZER_DESC,
            D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
            D3D11_VIEWPORT,
        },
        d3dcommon::{
            D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL,
            D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
        },
        errhandlingapi::GetLastError,
        libloaderapi::GetModuleHandleW,
        unknwnbase::IUnknown,
        winuser::{
            AdjustWindowRectEx, BeginPaint, CreateWindowExW, DefWindowProcW, DestroyWindow,
            DispatchMessageW, EndPaint, GetClientRect, GetSystemMetrics, LoadCursorW, LoadIconW,
            PeekMessageW, PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage,
            COLOR_WINDOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, IDI_APPLICATION, MSG,
            PAINTSTRUCT, PM_REMOVE, SW_SHOW, WM_CLOSE, WM_DESTROY, WM_PAINT, WM_QUIT, WNDCLASSEXW,
            WS_MAXIMIZE, WS_OVERLAPPEDWINDOW,
        },
    },
    Interface,
};

type HResult<A> = Result<A, HRESULT>;

const NUM_CONST_BUFFERS: usize = 3;
const TARGET_FPS: f32 = 60.0;
const MAX_TIME_STEP: Duration =
    Duration::from_nanos(((1.0 / TARGET_FPS) * 60_000_000_000.0) as u64);
const CB_APP: usize = 0;
const CB_FRAME: usize = 1;
const CB_OBJECT: usize = 2;
#[rustfmt::skip]
const CUBE: Cube = Cube {
    vertices: [
        VertexPosColor {
            position: XMFLOAT3 {
                x: -1.0, y: -1.0, z: -1.0,
            },
            color: XMFLOAT3 {
                x: 0.0, y: 0.0, z: 0.0,
            },
        },
        VertexPosColor {
            position: XMFLOAT3 {
                x: -1.0, y: 1.0, z: -1.0,
            },
            color: XMFLOAT3 {
                x: 0.0, y: 1.0, z: 0.0,
            },
        },
        VertexPosColor {
            position: XMFLOAT3 {
                x: 1.0, y: 1.0, z: -1.0,
            },
            color: XMFLOAT3 {
                x: 1.0, y: 1.0, z: 0.0,
            },
        },
        VertexPosColor {
            position: XMFLOAT3 {
                x: 1.0, y: -1.0, z: -1.0,
            },
            color: XMFLOAT3 {
                x: 1.0, y: 0.0, z: 0.0,
            },
        },
        VertexPosColor {
            position: XMFLOAT3 {
                x: -1.0, y: -1.0, z: 1.0,
            },
            color: XMFLOAT3 {
                x: 0.0, y: 0.0, z: 1.0,
            },
        },
        VertexPosColor {
            position: XMFLOAT3 {
                x: -1.0, y: 1.0, z: 1.0,
            },
            color: XMFLOAT3 {
                x: 0.0, y: 1.0, z: 1.0,
            },
        },
        VertexPosColor {
            position: XMFLOAT3 {
                x: 1.0, y: 1.0, z: 1.0,
            },
            color: XMFLOAT3 {
                x: 1.0, y: 1.0, z: 1.0,
            },
        },
        VertexPosColor {
            position: XMFLOAT3 {
                x: 1.0, y: -1.0, z: 1.0,
            },
            color: XMFLOAT3 {
                x: 1.0, y: 0.0, z: 1.0,
            },
        },
    ],
    indicies: [
        0, 1, 2, 0, 2, 3,
        4, 6, 5, 4, 7, 6,
        4, 5, 1, 4, 1, 0,
        3, 2, 6, 3, 6, 7,
        1, 5, 6, 1, 6, 2,
        4, 0, 3, 4, 3, 7,
    ],
};

pub(crate) struct App<'a> {
    //General
    width: u16,
    height: u16,
    window_class_name: String,
    window_name: String,
    window_handle: HWND,
    vsync: bool,
    //DX11 Specific
    d_device: &'a mut ID3D11Device,
    d_device_context: &'a mut ID3D11DeviceContext,
    d_swapchain: &'a mut IDXGISwapChain,
    d_render_target_view: &'a mut ID3D11RenderTargetView,
    d_depth_stencil_view: &'a mut ID3D11DepthStencilView,
    d_depth_stencil_buffer: &'a mut ID3D11Texture2D,
    d_depth_stencil_state: &'a mut ID3D11DepthStencilState,
    d_rasterizer_state: &'a mut ID3D11RasterizerState,
    d_viewport: D3D11_VIEWPORT,
    //Demo-Specific
    d_input_layout: &'a mut ID3D11InputLayout,
    d_vertex_buffer: &'a mut ID3D11Buffer,
    d_index_buffer: &'a mut ID3D11Buffer,
    d_vertex_shader: &'a mut ID3D11VertexShader,
    d_pixel_shader: &'a mut ID3D11PixelShader,
    d_constant_buffers: [&'a mut ID3D11Buffer; NUM_CONST_BUFFERS],
    d_projection_matrix: XMMatrix,
    d_view_matrix: XMMatrix,
    d_world_matrix: XMMatrix,
    t_previous: Instant,
    angle: f32,
}

struct VertexPosColor {
    color: XMFLOAT3,
    position: XMFLOAT3,
}
struct Cube {
    vertices: [VertexPosColor; 8],
    indicies: [u16; 36],
}
#[derive(Copy, Clone)]
pub(crate) enum WindowState {
    Windowed(u16, u16),
    Maximized,
    Fullscreen,
}
#[derive(Copy, Clone)]
pub(crate) struct Flags {
    pub vsync: bool,
    pub state: self::WindowState,
}
impl<'a> App<'a> {
    pub(crate) fn init(flags: self::Flags, class_name: String) -> HResult<Self> {
        let (width, height, window_name, window_handle, vsync) =
            Self::init_application(flags, &class_name)?;
        let (d_device, d_device_context, d_swapchain, client_width, client_height) =
            Self::init_device_and_swapchain(window_handle, flags)?;
        let d_render_target_view = Self::init_rtv(d_swapchain, d_device)?;
        let d_depth_stencil_buffer =
            Self::init_depth_stencil_buffer(client_width, client_height, d_device)?;
        let d_depth_stencil_view = Self::init_depth_stencil_view(d_device, d_depth_stencil_buffer)?;
        let d_depth_stencil_state = Self::init_depth_stencil_state(d_device)?;
        let d_rasterizer_state = Self::init_rasterizer_state(d_device)?;
        let d_viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: client_width as f32,
            Height: client_height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        //Demo now
        let [d_vertex_buffer, d_index_buffer] = Self::init_buffers(d_device)?;
        let mut d_constant_buffers = Self::init_const_buffers(d_device)?;
        let (vertex_data, d_vertex_shader, d_pixel_shader) = Self::load_shaders(d_device)?;
        let d_input_layout = Self::init_input_layout(d_device, vertex_data)?;
        let [d_projection_matrix, d_view_matrix, d_world_matrix] = Self::init_matrices(
            client_width,
            client_height,
            d_device_context,
            &mut d_constant_buffers,
        );
        let angle = 0.0;
        Ok(Self {
            //General
            width,
            height,
            window_name,
            window_handle,
            window_class_name: class_name,
            vsync,
            //DX11
            d_device,
            d_device_context,
            d_swapchain,
            d_render_target_view,
            d_depth_stencil_view,
            d_depth_stencil_buffer,
            d_depth_stencil_state,
            d_rasterizer_state,
            d_viewport,
            //Demo
            d_input_layout,
            d_vertex_buffer,
            d_index_buffer,
            d_vertex_shader,
            d_pixel_shader,
            d_constant_buffers,
            d_projection_matrix,
            d_view_matrix,
            d_world_matrix,
            t_previous: Instant::now(),
            angle,
        })
    }
    pub(crate) fn init_application(
        flags: self::Flags,
        class_name: &str,
    ) -> HResult<(u16, u16, String, HWND, bool)> {
        let h_instance = unsafe { GetModuleHandleW(ptr::null()) };
        let win_name = String::from("A Game");
        let window = utils::str_to_c16(&win_name);
        let class = utils::str_to_c16(class_name);
        let (win_width, win_height) = match flags.state {
            WindowState::Windowed(x, y) => (x as i32, y as i32),
            WindowState::Maximized => unsafe { (GetSystemMetrics(61), GetSystemMetrics(62)) },
            WindowState::Fullscreen => (CW_USEDEFAULT, CW_USEDEFAULT),
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
                WS_OVERLAPPEDWINDOW
                    | (match flags.state {
                        WindowState::Maximized => WS_MAXIMIZE,
                        _ => 0,
                    }),
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
            return Err(unsafe { utils::win32_to_hresult(GetLastError()) });
        }
        unsafe { ShowWindow(h_wnd, SW_SHOW) };
        Ok((
            a_width as u16,
            a_height as u16,
            win_name,
            h_wnd,
            flags.vsync,
        ))
    }

    pub(crate) fn init_device_and_swapchain<'b>(
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
                    Numerator: 0,
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
            BufferCount: 1,
            OutputWindow: window_handle,
            Windowed: if let WindowState::Windowed(_, _) = flags.state {
                TRUE
            } else {
                FALSE
            },
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
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
        let mut swapchain: MaybeUninit<&mut IDXGISwapChain> = MaybeUninit::zeroed();
        let mut device: MaybeUninit<&mut ID3D11Device> = MaybeUninit::zeroed();
        let mut immediate_context: MaybeUninit<&mut ID3D11DeviceContext> = MaybeUninit::zeroed();
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
                swapchain.as_mut_ptr() as *mut *mut _,
                device.as_mut_ptr() as *mut *mut _,
                &mut feature_level,
                immediate_context.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let swapchain = unsafe { swapchain.assume_init() };
        let device = unsafe { device.assume_init() };
        let immediate_context = unsafe { immediate_context.assume_init() };
        Ok((
            device,
            immediate_context,
            swapchain,
            client_width,
            client_height,
        ))
    }

    pub(crate) fn init_rtv<'b>(
        swapchain: &mut IDXGISwapChain,
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11RenderTargetView> {
        let mut back_buffer: MaybeUninit<&mut ID3D11Texture2D> = MaybeUninit::zeroed();
        let result = unsafe {
            swapchain.GetBuffer(
                0,
                &ID3D11Texture2D::uuidof(),
                back_buffer.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let back_buffer = unsafe { back_buffer.assume_init() };

        let mut rtv: MaybeUninit<&mut ID3D11RenderTargetView> = MaybeUninit::zeroed();
        let result = unsafe {
            device.CreateRenderTargetView(
                back_buffer as *mut _ as *mut _,
                ptr::null(),
                rtv.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let rtv = unsafe { rtv.assume_init() };
        Ok(rtv)
    }

    pub(crate) fn init_depth_stencil_buffer<'b>(
        client_width: i32,
        client_height: i32,
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11Texture2D> {
        let mut depth_stencil_buffer: MaybeUninit<&mut ID3D11Texture2D> = MaybeUninit::zeroed();
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
                depth_stencil_buffer.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let depth_stencil_buffer = unsafe { depth_stencil_buffer.assume_init() };
        Ok(depth_stencil_buffer)
    }

    pub(crate) fn init_depth_stencil_view<'b>(
        device: &mut ID3D11Device,
        depth_stencil_buffer: &mut ID3D11Texture2D,
    ) -> HResult<&'b mut ID3D11DepthStencilView> {
        let mut depth_stencil_view: MaybeUninit<&mut ID3D11DepthStencilView> =
            MaybeUninit::zeroed();
        let result = unsafe {
            device.CreateDepthStencilView(
                depth_stencil_buffer as *mut _ as *mut _,
                ptr::null(),
                depth_stencil_view.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let depth_stencil_view = unsafe { depth_stencil_view.assume_init() };
        Ok(depth_stencil_view)
    }

    pub(crate) fn init_depth_stencil_state<'b>(
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11DepthStencilState> {
        let mut depth_stencil_state: MaybeUninit<&mut ID3D11DepthStencilState> =
            MaybeUninit::zeroed();
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
            device.CreateDepthStencilState(
                &depth_stencil_state_desc,
                depth_stencil_state.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let depth_stencil_state = unsafe { depth_stencil_state.assume_init() };
        Ok(depth_stencil_state)
    }
    pub(crate) fn init_rasterizer_state<'b>(
        device: &mut ID3D11Device,
    ) -> HResult<&'b mut ID3D11RasterizerState> {
        let mut rasterizer_state: MaybeUninit<&mut ID3D11RasterizerState> = MaybeUninit::zeroed();
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
        let result = unsafe {
            device.CreateRasterizerState(
                &rasterizer_desc,
                rasterizer_state.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let rasterizer_state = unsafe { rasterizer_state.assume_init() };
        Ok(rasterizer_state)
    }
    //Demo now
    pub(crate) fn init_buffers<'b>(device: &ID3D11Device) -> HResult<[&'b mut ID3D11Buffer; 2]> {
        let mut vertex_buffer: MaybeUninit<&mut ID3D11Buffer> = MaybeUninit::zeroed();
        let vertex_buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: size_of::<VertexPosColor>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let resource_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: (&CUBE.vertices) as *const _ as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };
        let result = unsafe {
            device.CreateBuffer(
                &vertex_buffer_desc,
                &resource_data,
                vertex_buffer.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        }
        let vertex_buffer = unsafe { vertex_buffer.assume_init() };
        let mut index_buffer: MaybeUninit<&mut ID3D11Buffer> = MaybeUninit::zeroed();
        let index_buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: (size_of::<u16>() * 36) as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_INDEX_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let resource_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: (&CUBE.indicies) as *const _ as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };
        let result = unsafe {
            device.CreateBuffer(
                &index_buffer_desc,
                &resource_data,
                index_buffer.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        }
        let index_buffer = unsafe { index_buffer.assume_init() };
        Ok([vertex_buffer, index_buffer])
    }
    pub(crate) fn init_const_buffers<'b>(
        device: &ID3D11Device,
    ) -> HResult<[&'b mut ID3D11Buffer; 3]> {
        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: size_of::<XMMatrix>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: 0,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let mut const_buffers: [MaybeUninit<&mut ID3D11Buffer>; 3] = [
            MaybeUninit::zeroed(),
            MaybeUninit::zeroed(),
            MaybeUninit::zeroed(),
        ];
        let result = unsafe {
            device.CreateBuffer(
                &buffer_desc,
                ptr::null(),
                const_buffers[0].as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        }
        let result = unsafe {
            device.CreateBuffer(
                &buffer_desc,
                ptr::null(),
                const_buffers[1].as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        }
        let result = unsafe {
            device.CreateBuffer(
                &buffer_desc,
                ptr::null(),
                const_buffers[2].as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        }
        let const_buffers = unsafe { mem::transmute::<_, [&mut ID3D11Buffer; 3]>(const_buffers) };
        Ok(const_buffers)
    }
    pub(crate) fn load_shaders<'b>(
        device: &ID3D11Device,
    ) -> HResult<(&[u8], &'b mut ID3D11VertexShader, &'b mut ID3D11PixelShader)> {
        let vertex_data = include_bytes!(concat!(env!("OUT_DIR"), "\\VertexShader.cso"));
        let pixel_data = include_bytes!(concat!(env!("OUT_DIR"), "\\PixelShader.cso"));
        let mut vertex_shader: MaybeUninit<&mut ID3D11VertexShader> = MaybeUninit::zeroed();
        let mut pixel_shader: MaybeUninit<&mut ID3D11PixelShader> = MaybeUninit::zeroed();
        let result = unsafe {
            device.CreateVertexShader(
                vertex_data.as_ptr() as *const _,
                vertex_data.len(),
                ptr::null_mut(),
                vertex_shader.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let result = unsafe {
            device.CreatePixelShader(
                pixel_data.as_ptr() as *const _,
                pixel_data.len(),
                ptr::null_mut(),
                pixel_shader.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        };
        let vertex_shader = unsafe { vertex_shader.assume_init() };
        let pixel_shader = unsafe { pixel_shader.assume_init() };
        Ok((vertex_data, vertex_shader, pixel_shader))
    }
    pub(crate) fn init_input_layout<'b>(
        device: &ID3D11Device,
        vertex_data: &[u8],
    ) -> HResult<&'b mut ID3D11InputLayout> {
        let vertex_input_desc = [
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: "POSITION".as_ptr() as *const i8,
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: crate::offset_of!(VertexPosColor, position) as u32,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: "COLOR".as_ptr() as *const i8,
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: crate::offset_of!(VertexPosColor, color) as u32,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];
        let mut input_layout: MaybeUninit<&mut ID3D11InputLayout> = MaybeUninit::zeroed();
        let result = unsafe {
            device.CreateInputLayout(
                vertex_input_desc.as_ptr(),
                2,
                vertex_data.as_ptr() as *const _,
                vertex_data.len(),
                input_layout.as_mut_ptr() as *mut *mut _,
            )
        };
        if result != 0 {
            return Err(result);
        }
        let input_layout = unsafe { input_layout.assume_init() };
        Ok(input_layout)
    }
    pub(crate) fn init_matrices(
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
                constant_buffers[CB_APP] as *mut _ as *mut _,
                0,
                ptr::null(),
                &matrix as *const _ as *const _,
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
                constant_buffers[CB_FRAME] as *mut _ as *mut _,
                0,
                ptr::null(),
                &view_matrix as *const _ as *const _,
                0,
                0,
            )
        };
        let rotation_axis = XMVector::set(0.0, 1.0, 1.0, 0.0);
        let world_matrix = XMMatrixRotationAxis(rotation_axis.0, 0.0);
        unsafe {
            device_context.UpdateSubresource(
                constant_buffers[CB_OBJECT] as *mut _ as *mut _,
                0,
                ptr::null(),
                &world_matrix as *const _ as *const _,
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
    //Main loop
    pub(crate) fn run(&mut self) -> HRESULT {
        let mut msg: MaybeUninit<MSG> = MaybeUninit::zeroed();
        self.t_previous = Instant::now();
        while unsafe { (*msg.as_ptr()).message } != WM_QUIT {
            if unsafe { PeekMessageW(msg.as_mut_ptr(), ptr::null_mut(), 0, 0, PM_REMOVE) } == TRUE {
                unsafe {
                    TranslateMessage(msg.as_mut_ptr());
                    DispatchMessageW(msg.as_mut_ptr());
                }
            } else {
                let current_time = Instant::now();
                let mut delta_time = (current_time - self.t_previous).div_f32(1000.0);
                self.t_previous = current_time;
                delta_time = delta_time.min(MAX_TIME_STEP);

                self.update(delta_time.as_secs_f32());
                self.render();
            }
        }
        unsafe { (*msg.as_ptr()).wParam as i32 }
    }
    pub(crate) fn update(&mut self, delta_time: f32) {
        let eye_position = XMVector::set(0.0, 0.0, -10.0, 1.0);
        let focus_point = XMVector::set(0.0, 0.0, 0.0, 1.0);
        let up_direction = XMVector::set(0.0, 1.0, 0.0, 0.0);
        let view_matrix = XMMatrixLookAtLH(eye_position.0, focus_point.0, up_direction.0);
        unsafe {
            self.d_device_context.UpdateSubresource(
                self.d_constant_buffers[CB_FRAME] as *mut _ as *mut _,
                0,
                ptr::null(),
                &view_matrix as *const _ as *const _,
                0,
                0,
            )
        };
        self.angle += 90.0 * delta_time;
        let rotation_axis = XMVector::set(0.0, 1.0, 1.0, 0.0);
        let world_matrix = XMMatrixRotationAxis(rotation_axis.0, XMConvertToRadians(self.angle));
        unsafe {
            self.d_device_context.UpdateSubresource(
                self.d_constant_buffers[CB_OBJECT] as *mut _ as *mut _,
                0,
                ptr::null(),
                &world_matrix as *const _ as *const _,
                0,
                0,
            )
        };
    }
    pub(crate) fn clear(&mut self, color: [f32; 4], clear_depth: f32, clear_stencil: u8) {
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
    pub(crate) fn present(&mut self) {
        if self.vsync {
            unsafe {
                self.d_swapchain.Present(1, 0);
            }
        } else {
            unsafe {
                self.d_swapchain.Present(0, 0);
            }
        }
    }
    pub(crate) fn render(&mut self) {
        assert!(!(self.d_device_context as *mut ID3D11DeviceContext).is_null());
        assert!(!(self.d_device as *mut ID3D11Device).is_null());
        self.clear([0.3921569, 0.58431375, 0.9294119, 1.0], 1.0, 0);
        let vertex_stride = size_of::<VertexPosColor>();
        let offset = 0;
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
            self.d_device_context
                .DrawIndexed(CUBE.indicies.len() as u32, 0, 0);
            self.present();
        }
    }
}
impl<'a> Drop for App<'a> {
    fn drop(&mut self) {
        release!(self.d_constant_buffers[CB_OBJECT]);
        release!(self.d_constant_buffers[CB_FRAME]);
        release!(self.d_constant_buffers[CB_APP]);
        release!(self.d_index_buffer);
        release!(self.d_vertex_buffer);
        release!(self.d_input_layout);
        release!(self.d_vertex_shader);
        release!(self.d_pixel_shader);
        release!(self.d_depth_stencil_view);
        release!(self.d_render_target_view);
        release!(self.d_depth_stencil_buffer);
        release!(self.d_depth_stencil_state);
        release!(self.d_rasterizer_state);
        release!(self.d_swapchain);
        release!(self.d_device_context);
        release!(self.d_device);
    }
}
unsafe extern "system" fn wnd_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut paint_struct: MaybeUninit<PAINTSTRUCT> = MaybeUninit::zeroed();
            let _hdc = BeginPaint(h_wnd, paint_struct.as_mut_ptr());
            EndPaint(h_wnd, paint_struct.as_ptr());
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_CLOSE => {
            DestroyWindow(h_wnd);
            0
        }
        _ => DefWindowProcW(h_wnd, msg, w_param, l_param),
    }
}
