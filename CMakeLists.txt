cmake_minimum_required(VERSION 3.15)
project(lambdust VERSION 0.1.1 LANGUAGES C CXX)

# Set C and C++ standards
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Build options
option(BUILD_EXAMPLES "Build C/C++ integration examples" ON)
option(INSTALL_EXAMPLES "Install C/C++ integration examples" OFF)
option(BUILD_SHARED_LIBS "Build shared libraries" ON)

# Find required packages
find_package(PkgConfig QUIET)

# Rust library target (built externally)
add_library(lambdust_rust STATIC IMPORTED GLOBAL)

# Determine the Rust target directory and library name
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/debug")
    set(RUST_LIB_NAME "liblambdust.a")
else()
    set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/release")
    set(RUST_LIB_NAME "liblambdust.a")
endif()

# Set the imported library properties
set_target_properties(lambdust_rust PROPERTIES
    IMPORTED_LOCATION "${RUST_TARGET_DIR}/${RUST_LIB_NAME}"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

# Create an interface library for easier linking
add_library(lambdust INTERFACE)
target_link_libraries(lambdust INTERFACE lambdust_rust)
target_include_directories(lambdust INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

# Platform-specific libraries
if(WIN32)
    target_link_libraries(lambdust INTERFACE ws2_32 userenv bcrypt ntdll)
elseif(APPLE)
    target_link_libraries(lambdust INTERFACE "-framework Security" "-framework Foundation")
else()
    target_link_libraries(lambdust INTERFACE dl pthread m)
endif()

# Custom target to build Rust library
add_custom_target(build_rust_lib ALL
    COMMAND cargo build $<$<CONFIG:Release>:--release>
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust library"
    VERBATIM
)

# Make sure the Rust library is built before anything that depends on it
add_dependencies(lambdust_rust build_rust_lib)

# Configure pkg-config file
if(PkgConfig_FOUND)
    set(PREFIX ${CMAKE_INSTALL_PREFIX})
    set(VERSION ${PROJECT_VERSION})
    configure_file(
        "${CMAKE_CURRENT_SOURCE_DIR}/lambdust.pc.in"
        "${CMAKE_CURRENT_BINARY_DIR}/lambdust.pc"
        @ONLY
    )
endif()

# Build examples if requested
if(BUILD_EXAMPLES)
    add_subdirectory(examples/c_integration)
    add_subdirectory(examples/cpp_integration)
endif()

# Installation
include(GNUInstallDirs)

# Install header files
install(FILES include/lambdust.h include/lambdust_enhanced.h
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

# Install Rust library
install(FILES "${RUST_TARGET_DIR}/${RUST_LIB_NAME}"
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
        RENAME liblambdust.a)

# Install pkg-config file
if(PkgConfig_FOUND)
    install(FILES "${CMAKE_CURRENT_BINARY_DIR}/lambdust.pc"
            DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig)
endif()

# Install examples if requested
if(INSTALL_EXAMPLES)
    install(DIRECTORY examples/
            DESTINATION ${CMAKE_INSTALL_DOCDIR}/examples
            FILES_MATCHING 
            PATTERN "*.c" 
            PATTERN "*.cpp" 
            PATTERN "*.h"
            PATTERN "CMakeLists.txt")
endif()

# Export targets
install(TARGETS lambdust
        EXPORT lambdust-targets
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})

# Install the export set
install(EXPORT lambdust-targets
        FILE lambdust-targets.cmake
        NAMESPACE lambdust::
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/lambdust)

# Create and install package config file
include(CMakePackageConfigHelpers)

configure_package_config_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/cmake/lambdust-config.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/lambdust-config.cmake"
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/lambdust
)

write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/lambdust-config-version.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/lambdust-config.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/lambdust-config-version.cmake"
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/lambdust
)

# Print build information
message(STATUS "Lambdust CMake Configuration:")
message(STATUS "  Version: ${PROJECT_VERSION}")
message(STATUS "  Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  Build examples: ${BUILD_EXAMPLES}")
message(STATUS "  Install examples: ${INSTALL_EXAMPLES}")
message(STATUS "  Rust target dir: ${RUST_TARGET_DIR}")
message(STATUS "  Install prefix: ${CMAKE_INSTALL_PREFIX}")