cmake_minimum_required(VERSION 3.24)

project(knave-shell VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_program(CARGO_EXECUTABLE cargo REQUIRED)
find_program(WAYLAND_SCANNER_EXECUTABLE wayland-scanner REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(WaylandClient REQUIRED IMPORTED_TARGET wayland-client)
pkg_get_variable(WAYLAND_PROTOCOLS_DATADIR wayland-protocols pkgdatadir)
if(NOT WAYLAND_PROTOCOLS_DATADIR)
    message(FATAL_ERROR "wayland-protocols pkgdatadir was not found")
endif()
find_package(Threads REQUIRED)
find_package(Qt6 6.6 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 WaylandClient)
find_package(Qt6WaylandClientPrivate REQUIRED)
find_package(Qt6WaylandScannerTools REQUIRED)

qt_standard_project_setup(REQUIRES 6.6)
qt_policy(SET QTP0004 NEW)

if(NOT TARGET Wayland::Scanner)
    add_executable(Wayland::Scanner IMPORTED)
    set_target_properties(Wayland::Scanner PROPERTIES
        IMPORTED_LOCATION "${WAYLAND_SCANNER_EXECUTABLE}"
    )
endif()

set(CORE_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo")
set(CORE_ARCHIVE "${CORE_TARGET_DIR}/debug/libknave_shell_core.a")
file(GLOB_RECURSE CORE_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_SOURCE_DIR}/core/src/*.rs"
    "${CMAKE_SOURCE_DIR}/core/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/Cargo.lock"
)
add_custom_command(
    OUTPUT "${CORE_ARCHIVE}"
    COMMAND "${CMAKE_COMMAND}" -E env "CARGO_TARGET_DIR=${CORE_TARGET_DIR}"
            "${CARGO_EXECUTABLE}" build --locked --manifest-path "${CMAKE_SOURCE_DIR}/core/Cargo.toml"
    DEPENDS ${CORE_SOURCES}
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    COMMENT "Building knave-shell-core"
    VERBATIM
)
add_custom_target(knave-shell-core-build DEPENDS "${CORE_ARCHIVE}")
add_library(knave-shell-core STATIC IMPORTED GLOBAL)
set_target_properties(knave-shell-core PROPERTIES IMPORTED_LOCATION "${CORE_ARCHIVE}")
add_dependencies(knave-shell-core knave-shell-core-build)

qt_add_executable(knave-shell
    app/main.cpp
    app/shellservice.cpp
    app/shellservice.h
)
qt_add_qml_module(knave-shell
    URI KnaveShell
    VERSION 1.0
    QML_FILES
        qml/Bar.qml
        qml/Overview.qml
)
target_include_directories(knave-shell PRIVATE "${CMAKE_SOURCE_DIR}/core/include")
target_link_libraries(knave-shell PRIVATE
    knave-shell-core
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Threads::Threads
    ${CMAKE_DL_LIBS}
    m
)
set_target_properties(knave-shell PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)

add_library(knave-layer-shell-plugin MODULE
    layer-shell/plugin.cpp
    layer-shell/layershellintegration.cpp
    layer-shell/layershellintegration.h
    layer-shell/knave-layer-shell.json
)
qt_generate_wayland_protocol_client_sources(knave-layer-shell-plugin
    FILES
        "${CMAKE_SOURCE_DIR}/protocols/wlr-layer-shell-unstable-v1.xml"
        "${WAYLAND_PROTOCOLS_DATADIR}/stable/xdg-shell/xdg-shell.xml"
)
target_link_libraries(knave-layer-shell-plugin PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::WaylandClientPrivate
    PkgConfig::WaylandClient
)
set_target_properties(knave-layer-shell-plugin PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/plugins/wayland-shell-integration"
)

install(TARGETS knave-shell RUNTIME DESTINATION bin)
install(TARGETS knave-layer-shell-plugin
    LIBRARY DESTINATION lib/knave-shell/plugins/wayland-shell-integration
)
