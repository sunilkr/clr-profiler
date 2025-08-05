## Usage

This project is configure by default to generate 32bit binaries (see [config.toml](/.cargo/config.toml)). `cargo build [--release]` builds 2 important binaries in `target\debug` (or `target\release` in case of `--release` option).

1. `test_profilers.dll`: DLL with [profiling logic](/test_profilers/src/basic_integration_test.rs), loaded by .NET runtime.
1. `profile.exe`: Utility to launch target with profiling environment. This expects that `test_profilers.dll` is present in same directory as this executable.

Launch command: `profile.exe <target>`


## CLR Profiling flow:
1. Some COM client (CLR in this case) calls `DllGetClassObject`, which populates a pointer (`[out] **ppv`) to an instance of a struct that adheres to `IClassFactory`.
1. The COM client then calls `CreateInstance` on the `IClassFactory` that it now has a handle to. This populates a pointer ([out] parameter) to an instance of a struct that adheres to `ICorProfilerCallback9`.
1. The COM client calls `Initialize` on the `ICorProfilerCallback` instance and provides an instance of `ICorProfilerInfo`, which profiler can use to call into .NET runtime.
1. Profiler instance calls `SetEventMask` on ProfilerInfo instance to subscribe to the events its is interested in.
1. Based on the EventMask provided in last call, CLR calls methods defined by `ICorProfilerCallback` interface.


Some Helpful Resources:
- Docs on some of the weird Windows specific types (LPVOID, DWORD, HINSTANCE, etc...)
  - https://en.wikibooks.org/wiki/Windows_Programming/Handles_and_Data_Types

## Some Links

- https://github.com/DataDog/dd-trace-dotnet
- https://docs.datadoghq.com/tracing/setup/dotnet/?tab=netcoreonlinux
- https://docs.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/icorprofilercallback2-interface
- https://docs.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/setting-up-a-profiling-environment
- https://github.com/microsoft/clr-samples/tree/master/ProfilingAPI
- https://github.com/microsoft/BPerf
- https://stackoverflow.com/questions/840501/how-do-function-pointers-in-c-work
- https://www.codeguru.com/cpp/com-tech/activex/misc/article.php/c5509/Simplifying-the-Concept-of-COM.htm
- https://docs.microsoft.com/en-us/windows/win32/com/implementing-iclassfactory
- https://docs.microsoft.com/en-us/windows/win32/api/unknwn/
- https://github.com/Rantanen/intercom/blob/master/intercom-common/src/attributes/com_library.rs#L87
- https://stackoverflow.com/questions/26117197/create-interface-to-c-function-pointers-in-rust
- https://www.reddit.com/r/rust/comments/4w6vjt/allocating_a_raw_double_pointer_in_rust/
- https://newrustacean.com/show_notes/e031/index.html
- http://jakegoulding.com/rust-ffi-omnibus/objects/
- https://docs.rs/libc/0.2.63/libc/enum.c_void.html
- https://docs.rs/winapi/0.3.8/winapi/ctypes/type.c_long.html
- https://docs.rs/intercom/0.2.0/intercom/struct.GUID.html
- https://opensource.stackexchange.com/questions/5143/how-to-mark-a-copied-apache-v2-piece-of-code
- https://www.codeproject.com/Articles/13601/COM-in-plain-C#CLASS
- https://gankra.github.io/blah/rust-layouts-and-abis/
- https://docs.rs/intercom/0.2.0/src/intercom/combox.rs.html#105-134
- https://learn.microsoft.com/en-us/archive/msdn-magazine/2003/september/write-msil-code-on-the-fly-with-the-net-framework-profiling-api
- https://github.com/microsoft/clr-samples/blob/master/ProfilingAPI/ReJITEnterLeaveHooks/ILRewriter.cpp

## TODO

- Create facility to run tests against multiple .net versions. For example, here is linux setup for [side-by-side .net core installations](https://www.hanselman.com/blog/SideBySideUserScopedNETCoreInstallationsOnLinuxWithDotnetinstallsh.aspx).
- A potential source of bugs in the `ProfilerInfo` struct, is when an `S_OK` hresult is returned, but one of the out pointer parameters can be null. These need explicitly checked if they are null and wrapped in an `Option`. If we just dereference, this is undefined behavior! The Microsoft documentation is spotty on which out parameters can be null. Sometimes it is mentioned in the remarks.

