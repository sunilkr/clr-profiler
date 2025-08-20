use clr_profiler::{
    ffi::{AssemblyID, ClassFactory, CorOpenFlags, FunctionID, ModuleID, COR_PRF_MONITOR, HRESULT, LPVOID, REFCLSID, REFIID}, 
    ClrProfiler, CorProfilerCallback, CorProfilerCallback2, CorProfilerCallback3, CorProfilerCallback4, CorProfilerCallback5, 
    CorProfilerCallback6, CorProfilerCallback7, CorProfilerCallback8, CorProfilerCallback9, 
    CorProfilerInfo, CorProfilerInfo3,
    MetadataImportTrait, ProfilerInfo
};
use log::{debug, info, warn};
use log_init::init_logging;
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

    fn with_clsid(clsid: Uuid) -> Self {
        Self { clsid: clsid, profiler_info: None }
    }
}

impl ClrProfiler for Profiler {
    fn new() -> Profiler {
        Profiler {
            clsid: Uuid::parse_str(PROFILER_UUID).unwrap(),
            profiler_info: None,
        }
    }
    fn clsid(&self) -> &Uuid {
        &self.clsid
    }
}

impl CorProfilerCallback for Profiler {
    fn initialize(&mut self, profiler_info: ProfilerInfo) -> Result<(), HRESULT> {
        // Initialize ICorProfilerInfo reference
        self.profiler_info = Some(profiler_info);

        // Set the event mask
        self.profiler_info()
            .set_event_mask(COR_PRF_MONITOR::COR_PRF_ALL)?; // COR_PRF_MONITOR_JIT_COMPILATION
            // .set_event_mask(
            //     COR_PRF_MONITOR::COR_PRF_DISABLE_ALL_NGEN_IMAGES
            //     | COR_PRF_MONITOR::COR_PRF_MONITOR_ASSEMBLY_LOADS 
            //     | COR_PRF_MONITOR::COR_PRF_MONITOR_MODULE_LOADS
            //     | COR_PRF_MONITOR::COR_PRF_MONITOR_JIT_COMPILATION
            // )?;

        Ok(())
    }

    fn assembly_load_finished(&mut self, assembly_id: AssemblyID, hr_status: HRESULT) -> Result<(), HRESULT> {
        let assembly_info = self.profiler_info().get_assembly_info(assembly_id)?;
        info!("assembly {} finished loading with status {:#x}", assembly_info.name, hr_status);
        Ok(())
    }

    fn module_load_finished(&mut self, module_id: ModuleID, hr_status: HRESULT) -> Result<(), HRESULT> {
        let module_info = self.profiler_info().get_module_info_2(module_id)?;
        info!("moudle load finished with status {} : {:#?}", hr_status, module_info);
        Ok(())
    }

    fn jit_compilation_started(&mut self, function_id: FunctionID, _is_safe_to_block: bool) -> Result<(), HRESULT> {
        // Get function metadata from function id.
        let function_info = self.profiler_info().get_function_info(function_id)?;
        
        // Get metadata interface for module of this function.
        // TODO: Store this instance??
        let module_metadata = self.profiler_info()
            .get_module_metadata(function_info.module_id, CorOpenFlags::ofRead)?;

        // Get method properties using module metadata interface.
        let method_props = module_metadata.get_method_props(function_info.token)?;
        
        // Get class (Type Definition) properties using module metadata interface.
        let class_props = module_metadata.get_type_def_props(method_props.class_token)?;

        let qualified_method_name = format!("{}.{}", class_props.name, method_props.name);
        info!("jit compilation started for {qualified_method_name}");

        // if method_props.name == "TMethod" || method_props.name == "FMethod" {
        //     let il_body = self
        //     .profiler_info()
        //     .get_il_function_body(function_info.module_id, function_info.token)?;
        //     // let bytes = unsafe {
        //     //     slice::from_raw_parts(il_body.method_header, il_body.method_size as usize)
        //     // };
        //     let mut method =
        //         Method::new(il_body.method_header, il_body.method_size).or(Err(E_FAIL))?;
        //     println!("{:#?}", method);
        //     let il = vec![nop()];
        // }
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

#[unsafe(no_mangle)]
unsafe extern "system" fn DllGetClassObject( rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    println!("[PROF_DEBUG] In DllGetClassObject");
    init_logging();

    debug!("DllGetClassObject called");
    
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
            warn!("failed to covert {guid:?} to UUID; err: {err}");
            warn!("creating profiler with default GUID {{{PROFILER_UUID}}}");
            Profiler::new()
        }
    };
    
    let class_factory: &mut ClassFactory<Profiler> = ClassFactory::new(profiler);

    // Initialize [out]ppv pointer with IClassFactory instance.
    unsafe { class_factory.QueryInterface(riid, ppv) }
}
