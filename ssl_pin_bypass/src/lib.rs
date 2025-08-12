use clr_profiler::{
    cil::{ldc_i4_1, ret, Method, MethodHeader, TinyMethodHeader},
    ffi::{ClassFactory, CorOpenFlags, FunctionID, COR_PRF_MONITOR, E_FAIL, HRESULT, LPVOID, REFCLSID, REFIID}, ClrProfiler, CorProfilerCallback, CorProfilerCallback2, CorProfilerCallback3,
    CorProfilerCallback4, CorProfilerCallback5, CorProfilerCallback6, CorProfilerCallback7,
    CorProfilerCallback8, CorProfilerCallback9, CorProfilerInfo, MetadataImportTrait, ProfilerInfo,
};
use log::{debug, error, info, trace, warn};
use uuid::Uuid;

const PROFILER_UUID: &str = "DF63A541-5A33-4611-8829-F4E495985EE3";

#[derive(Clone)]
struct Profiler {
    clsid: Uuid,
    profiler_info: Option<ProfilerInfo>,
}

impl Profiler {
    fn profiler_info(&self) -> &ProfilerInfo {
        self.profiler_info.as_ref().unwrap()
    }

    fn with_clsid(clsid: Uuid) -> Self{
        Self { clsid: clsid, profiler_info: None }
    }
}

impl ClrProfiler for Profiler {
    fn new() -> Profiler {
        Profiler {
            clsid: Uuid::parse_str(PROFILER_UUID).unwrap(), //Should not result in error.
            profiler_info: None,
        }
    }
    fn clsid(&self) -> &Uuid {
        &self.clsid
    }
}

impl CorProfilerCallback for Profiler {
    fn initialize(&mut self, profiler_info: ProfilerInfo) -> Result<(), HRESULT> {
        info!("profiler initialized");
        
        // Initialize ICorProfilerInfo reference    
        self.profiler_info = Some(profiler_info);

        // Set the event mask
        self.profiler_info()
            .set_event_mask(COR_PRF_MONITOR::COR_PRF_ALL)?; // COR_PRF_MONITOR_JIT_COMPILATION

        match self.profiler_info().get_event_mask() {
            Ok(mask) => debug!("current event mask: {mask:?}"),
            Err(hresult) => warn!("failed to get current event mask, HRESULT = {hresult:#x}"),
        } 

        Ok(())
    }

    fn jit_compilation_started(&mut self, function_id: FunctionID, _is_safe_to_block: bool) -> Result<(), HRESULT> {
        let function_info = self.profiler_info().get_function_info(function_id)?;
        let module_metadata = self
            .profiler_info()
            .get_module_metadata(function_info.module_id, CorOpenFlags::ofRead)?;
        let method_props = module_metadata.get_method_props(function_info.token)?;
        //println!("[PROF] started JIT compilation for {}", method_props.name);
        
        if method_props.name == "userCertValidationCallbackWrapper" {
            let class_props = module_metadata.get_type_def_props(method_props.class_token)?;
            let qualified_method_name = format!("{}.{}", class_props.name, method_props.name);
            info!("inspecting {qualified_method_name}()");
            
            debug!("{class_props:#?}");
            debug!("{method_props:#?}");

            let il_body = self
                .profiler_info()
                .get_il_function_body(function_info.module_id, function_info.token)?;
            
            // let bytes = unsafe {
            //     slice::from_raw_parts(il_body.method_header, il_body.method_size as usize)
            // };
            
            let method = Method::new(il_body.method_header, il_body.method_size).or(Err(E_FAIL))?;
            info!("{:#?}", method.method_header);
            let body = method.instructions.iter()
                .map(|inst| format!("{inst}"))
                .collect::<Vec<String>>()
                .join("\n    ");
            debug!("body: \n{{\n    {body}\n}}");
            
            if method.sections.len() > 0 {
                info!("sections: {:#?}", method.sections);
            }

            info!("attemtpting to replace body of {qualified_method_name}()");
            
            // TODO: Figure out a way to create Methods from Instructions.
            let new_method = Method{
                method_header: MethodHeader::Tiny(TinyMethodHeader{code_size: 2}),
                instructions: vec![ldc_i4_1(), ret()],
                sections: vec![]
            };

            let method_bytes = new_method.into_bytes();
            // TODO: figure out how to use ILFunctionBodyAllocator
            self.profiler_info().set_il_function_body(function_info.module_id, function_info.token, method_bytes.as_ptr())?;
            info!("function body replaced");
            
        }
        
        module_metadata.release();
        // 1. Modify method header
        // 2. Add a prologue
        // 3. Add an epilogue
        // 4. Modify SEH tables
        Ok(())
    }
}

impl CorProfilerCallback2 for Profiler {}
impl CorProfilerCallback3 for Profiler {}
impl CorProfilerCallback4 for Profiler {}
impl CorProfilerCallback5 for Profiler {}
impl CorProfilerCallback6 for Profiler {}
impl CorProfilerCallback7 for Profiler {}
impl CorProfilerCallback8 for Profiler {}
impl CorProfilerCallback9 for Profiler {}

//register!(Profiler);

#[unsafe(no_mangle)]
unsafe extern "system" fn DllGetClassObject(
    rclsid: REFCLSID,
    riid: REFIID,
    ppv: *mut LPVOID,
    ) -> HRESULT {
    
    //println!("[PROF_DEBUG] In DllGetClassObject");

    init_logging();

    trace!("in DllGetClassObject");
    unsafe {
        debug!("*rclsid = {}, *riid = {}", *rclsid, *riid);
    }
    
    let guid = unsafe { *rclsid };
    let profiler = match Uuid::from_fields(guid.data1, guid.data2, guid.data3, &guid.data4) {
        Ok(uuid) => {
            info!("creating profiler with UUID: {{{}}}", uuid.to_hyphenated());
            Profiler::with_clsid(uuid)
        },
        Err(err) => {
            error!("failed to covert {guid:?} to UUID; err: {err}");
            warn!("creating profiler with default GUID {{{PROFILER_UUID}}}");
            Profiler::new()
        }
    };

    //let profiler = Profiler::new(rclsid);
    //let clsid = GUID::from(*profiler.clsid());
    
    let class_factory: &mut ClassFactory<Profiler> = ClassFactory::new(profiler);
    unsafe { class_factory.QueryInterface(riid, ppv) }
}


fn init_logging() {
    match simple_logger::SimpleLogger::new()
    .with_colors(true)
    .with_level(log::LevelFilter::Info)
    .without_timestamps()
    .init() {
        Ok(_) => {
            info!("logging initialized");
        }
        Err(err) => {
            eprintln!("failed to initialize logging; err: {err}");
        }
    }
}

