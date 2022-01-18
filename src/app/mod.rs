mod cubes;
mod directx11_init;
mod game_init;
mod hid;
mod run;
mod ui;

use crate::utils::{self, Coord};
use directx_math::{
    XMConvertToRadians, XMMatrix, XMMatrixLookAtLH, XMMatrixLookToLH, XMMatrixPerspectiveFovLH,
    XMMatrixRotationAxis, XMMatrixRotationX, XMMatrixRotationY, XMVector, XMVector3Transform,
    XMVectorSet,
};
use png::{Decoder, DecodingError, OutputInfo};
use std::{
    cell::{Cell, RefCell},
    f32::consts::{FRAC_PI_2, PI, TAU},
    fs::OpenOptions,
    io,
    mem::{size_of, MaybeUninit},
    panic::{catch_unwind, resume_unwind},
    path::Path,
    ptr,
    time::{Duration, Instant},
};
use winapi::{
    shared::{
        dxgi::{IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_FLIP_DISCARD},
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
            DispatchMessageW, EndPaint, GetClientRect, GetRawInputData, GetSystemMetrics,
            LoadCursorW, LoadIconW, PeekMessageW, PostQuitMessage, RegisterClassExW,
            RegisterRawInputDevices, ShowWindow, TranslateMessage, COLOR_WINDOW, CS_HREDRAW,
            CS_VREDRAW, CW_USEDEFAULT, HRAWINPUT, IDC_ARROW, IDI_APPLICATION, MOUSE_MOVE_ABSOLUTE,
            MSG, PAINTSTRUCT, PM_REMOVE, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RID_INPUT,
            RIM_TYPEKEYBOARD, RIM_TYPEMOUSE, RI_KEY_BREAK, SW_SHOW, VK_ESCAPE, WM_CLOSE, WM_CREATE,
            WM_DESTROY, WM_INPUT, WM_KEYDOWN, WM_KEYUP, WM_PAINT, WM_QUIT, WM_SYSKEYDOWN,
            WM_SYSKEYUP, WNDCLASSEXW, WS_MAXIMIZE, WS_OVERLAPPEDWINDOW, WS_POPUP,
        },
    },
    Interface,
};

type HResult<A> = Result<A, HRESULT>;

const NUM_CONST_BUFFERS: usize = 3;
const CB_APP: usize = 0;
const CB_FRAME: usize = 1;
const CB_OBJECT: usize = 2;
thread_local! {
    static KEYS: RefCell<[u16; 16]> = RefCell::new([0; 16]);
    static SYSKEYS: RefCell<[u16; 16]> = RefCell::new([0; 16]);
    static MOUSE: Cell<Position> = Cell::new(Position{x:0,y:0});
}
const TARGET_FPS: f32 = 60.0;
const MAX_TIME_STEP: Duration =
    Duration::from_nanos(((1.0 / TARGET_FPS) * 60_000_000_000.0) as u64);
//#[cfg(debug_assertions)]
const VERTEX_SHADER_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\VertexShader.cso"));
//#[cfg(debug_assertions)]
const PIXEL_SHADER_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\PixelShader.cso"));

const MAIN_UI: usize = 0;

#[derive(Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}
pub struct App<'a> {
    //General
    window_class_name: String,
    h_wnd: HWND,
    flags: self::Flags,
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
    //Game
    d_input_layout: &'a mut ID3D11InputLayout,
    d_vertex_buffer: &'a mut ID3D11Buffer,
    d_index_buffer: &'a mut ID3D11Buffer,
    d_vertex_shader: &'a mut ID3D11VertexShader,
    d_pixel_shader: &'a mut ID3D11PixelShader,
    d_constant_buffers: [&'a mut ID3D11Buffer; NUM_CONST_BUFFERS],
    projection_matrix: XMMatrix,
    view_matrix: XMMatrix,
    camera: (XMVector, XMVector),
    world_matrix: XMMatrix,
    pub t_previous: Instant,
    cubes: cubes::Cubes,
    assets: Vec<Asset>,
    state: State,
    meu_ids: Vec<(u16, Menu)>,
}
#[derive(Copy, Clone)]
pub enum WindowState {
    Windowed(u16, u16),
    Maximized,
    Fullscreen,
}
#[derive(Copy, Clone)]
pub struct Flags {
    pub vsync: bool,
    pub state: self::WindowState,
}
pub struct Asset {
    buf: Vec<u8>,
    info: OutputInfo,
}
pub enum State {
    Loading,
    MainMenu(u16),
    InGame(u16),
}
#[non_exhaustive]
pub enum Menu {
    Listmeny(ListMenu),
}
pub struct ListMenu {
    title: String,
    buttons: Button,
}
pub struct Button {
    active: bool,
    text: String,
    on_click: Option<u16>,
}
impl Clone for Asset {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf.clone(),
            info: OutputInfo { ..self.info },
        }
    }
}
impl<'a> App<'a> {
    pub fn init(flags: self::Flags, class_name: &str, h_wnd: HWND) -> HResult<Self> {
        let (d_device, d_device_context, d_swapchain, client_width, client_height) =
            Self::init_device_and_swapchain(h_wnd, flags)?;
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
        //Game now
        let cubes = cubes::Cubes::new_list(vec![
            Coord { x: -3, y: 0, z: 0 },
            Coord { x: -2, y: 0, z: 0 },
            Coord { x: -2, y: 1, z: 1 },
            Coord { x: -1, y: 0, z: 0 },
            Coord { x: -1, y: 0, z: 1 },
            Coord { x: -1, y: 1, z: 1 },
        ])
        .unwrap();
        let (verticies, indicies) = cubes.to_vertices(384, 312);
        let [d_vertex_buffer, d_index_buffer] =
            Self::init_buffers(d_device, &verticies, &indicies)?;
        let mut d_constant_buffers = Self::init_const_buffers(d_device)?;
        let (d_vertex_shader, d_pixel_shader) = Self::load_shaders(d_device)?;
        let d_input_layout = Self::init_input_layout(d_device)?;
        let [projection_matrix, view_matrix, world_matrix] = Self::init_matrices(
            client_width,
            client_height,
            d_device_context,
            &mut d_constant_buffers,
        );
        let camera = (
            XMVector::set(0.0, 0.0, -10.0, 0.0),
            XMVector::set(0.0, 0.0, 1.0, 0.0),
        );
        let assets = Self::load_assets()?;
        Ok(Self {
            //General
            h_wnd,
            window_class_name: class_name.into(),
            flags,
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
            //Game
            d_input_layout,
            d_vertex_buffer,
            d_index_buffer,
            d_vertex_shader,
            d_pixel_shader,
            d_constant_buffers,
            projection_matrix,
            view_matrix,
            camera,
            world_matrix,
            t_previous: Instant::now(),
            cubes,
            assets,
            state: State::MainMenu(0),
        })
    }
    //Main loop
    pub fn run(&mut self) -> HRESULT {
        //Loop
        let mut msg: MaybeUninit<MSG> = MaybeUninit::zeroed();
        let mut indicies = 0;
        while unsafe { (*msg.as_ptr()).message } != WM_QUIT {
            if unsafe { PeekMessageW(msg.as_mut_ptr(), ptr::null_mut(), 0, 0, PM_REMOVE) } == TRUE {
                unsafe {
                    TranslateMessage(msg.as_mut_ptr());
                    if DispatchMessageW(msg.as_mut_ptr()) == -2147483648 {
                        resume_unwind(Box::new("window procedure panicked"));
                    }
                }
            } else {
                let current_time = Instant::now();
                let mut delta_time = current_time - self.t_previous;
                self.t_previous = current_time;
                let min = delta_time.min(MAX_TIME_STEP);
                delta_time = min;

                let indicies_now = self.hid(delta_time.as_secs_f32());
                if indicies_now < 0 {
                    continue;
                } else if indicies_now > 0 {
                    indicies = indicies_now;
                }
                self.update();
                self.render(indicies);
            }
        }
        0
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
    catch_unwind(|| {
        match msg {
            //Start
            WM_CREATE => {
                let rid = [
                    RAWINPUTDEVICE {
                        usUsagePage: 0x01,
                        usUsage: 0x02,
                        dwFlags: 0,
                        hwndTarget: std::ptr::null_mut(),
                    },
                    RAWINPUTDEVICE {
                        usUsagePage: 0x01,
                        usUsage: 0x06,
                        dwFlags: 0,
                        hwndTarget: std::ptr::null_mut(),
                    },
                ];
                if RegisterRawInputDevices(rid.as_ptr(), 2, size_of::<RAWINPUTDEVICE>() as u32)
                    == FALSE
                {
                    eprintln!("Failed to register devices: {:X}", GetLastError());
                }
                0
            }
            //During
            WM_PAINT => {
                let mut paint_struct: MaybeUninit<PAINTSTRUCT> = MaybeUninit::zeroed();
                let _hdc = BeginPaint(h_wnd, paint_struct.as_mut_ptr());
                EndPaint(h_wnd, paint_struct.as_ptr());
                0
            }
            WM_INPUT => {
                let mut dw_size = 0;
                GetRawInputData(
                    l_param as HRAWINPUT,
                    RID_INPUT,
                    ptr::null_mut(),
                    <*mut _>::cast(&mut dw_size as *mut _),
                    size_of::<RAWINPUTHEADER>() as u32,
                );
                let mut data_buf = vec![0u8; dw_size];
                let data_slice = data_buf.as_mut_slice();
                let data = read_input(l_param, data_slice, dw_size);
                if data.header.dwType == RIM_TYPEMOUSE {
                    let mouse = data.data.mouse();
                    if (mouse.usFlags & MOUSE_MOVE_ABSOLUTE) == MOUSE_MOVE_ABSOLUTE {
                        MOUSE.with(|m| {
                            m.set(Position {
                                x: mouse.lLastX,
                                y: mouse.lLastY,
                            })
                        });
                    } else {
                        MOUSE.with(|m| {
                            let p = m.get();
                            m.set(Position {
                                x: p.x + mouse.lLastX,
                                y: p.y + mouse.lLastY,
                            })
                        });
                    }
                } else if data.header.dwType == RIM_TYPEKEYBOARD {
                    let keyboard = data.data.keyboard();
                    if keyboard.Flags as u32 & RI_KEY_BREAK == RI_KEY_BREAK {
                        match keyboard.Message {
                            WM_KEYDOWN | WM_KEYUP => {
                                set_key(false, false, keyboard.VKey);
                            }
                            WM_SYSKEYDOWN | WM_SYSKEYUP => {
                                set_key(true, false, keyboard.VKey);
                            }
                            _ => {}
                        }
                    } else {
                        match keyboard.Message {
                            WM_KEYDOWN | WM_KEYUP => {
                                set_key(false, true, keyboard.VKey);
                            }
                            WM_SYSKEYDOWN | WM_SYSKEYUP => {
                                set_key(true, true, keyboard.VKey);
                            }
                            _ => {}
                        }
                    }
                } else {
                    println!("Other...")
                }
                0
            }
            //End
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
    })
    .unwrap_or(-2147483648)
}
unsafe fn read_input(l_param: LPARAM, data_buf: &mut [u8], mut dw_size: usize) -> &mut RAWINPUT {
    GetRawInputData(
        l_param as HRAWINPUT,
        RID_INPUT,
        data_buf.as_mut_ptr() as *mut _,
        &mut dw_size as *mut _ as *mut _,
        size_of::<RAWINPUTHEADER>() as u32,
    );
    &mut *<*mut _>::cast(data_buf.as_mut_ptr())
}
fn set_key(sys: bool, down: bool, keycode: u16) {
    if keycode < 256 {
        let bit = keycode % 16;
        let slot = keycode / 16;
        if sys {
            SYSKEYS.with(|k| {
                let mut x = k.borrow_mut();
                if down {
                    x[slot as usize] |= 1 << bit;
                } else {
                    x[slot as usize] &= !(1 << bit);
                }
            })
        } else {
            KEYS.with(|k| {
                let mut x = k.borrow_mut();
                if down {
                    x[slot as usize] |= 1 << bit;
                } else {
                    x[slot as usize] &= !(1 << bit);
                }
            })
        }
    }
}
pub fn io_error(error: io::Error) -> HRESULT {
    match error.kind() {
        io::ErrorKind::NotFound => -2147024894,
        io::ErrorKind::PermissionDenied => -2147024891,
        io::ErrorKind::ConnectionRefused => -2147023671,
        io::ErrorKind::ConnectionReset => -2147012865,
        io::ErrorKind::ConnectionAborted => -2147023660,
        io::ErrorKind::NotConnected => -2147022646,
        io::ErrorKind::AddrInUse => -2147019839,
        io::ErrorKind::AddrNotAvailable => -2147019860,
        io::ErrorKind::BrokenPipe => -2147024787,
        io::ErrorKind::AlreadyExists => -2147024713,
        io::ErrorKind::WouldBlock => -2147014861,
        io::ErrorKind::InvalidInput => -2147024809,
        io::ErrorKind::InvalidData => -2147024883,
        io::ErrorKind::TimedOut => -2147024880,
        io::ErrorKind::WriteZero => -2147024867,
        io::ErrorKind::Interrupted => -2146233063,
        io::ErrorKind::Unsupported => -2147467263,
        io::ErrorKind::UnexpectedEof => -1072896679,
        io::ErrorKind::OutOfMemory => -2147024882,
        _ => -1,
    }
}
