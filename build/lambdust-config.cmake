
####### Expanded from @PACKAGE_INIT@ by configure_package_config_file() #######
####### Any changes to this file will be overwritten by the next CMake run ####
####### The input file was lambdust-config.cmake.in                            ########

get_filename_component(PACKAGE_PREFIX_DIR "${CMAKE_CURRENT_LIST_DIR}/../../../" ABSOLUTE)

macro(set_and_check _var _file)
  set(${_var} "${_file}")
  if(NOT EXISTS "${_file}")
    message(FATAL_ERROR "File or directory ${_file} referenced by variable ${_var} does not exist !")
  endif()
endmacro()

macro(check_required_components _NAME)
  foreach(comp ${${_NAME}_FIND_COMPONENTS})
    if(NOT ${_NAME}_${comp}_FOUND)
      if(${_NAME}_FIND_REQUIRED_${comp})
        set(${_NAME}_FOUND FALSE)
      endif()
    endif()
  endforeach()
endmacro()

####################################################################################

# Lambdust CMake configuration file
# This file is used by find_package(lambdust) to locate the Lambdust library

include(CMakeFindDependencyMacro)

# Find dependencies
if(WIN32)
    # Windows-specific dependencies are handled by target_link_libraries
elseif(APPLE)
    # macOS-specific dependencies are handled by target_link_libraries
else()
    # Linux/Unix dependencies
    find_dependency(Threads REQUIRED)
endif()

# Include the targets file
include("${CMAKE_CURRENT_LIST_DIR}/lambdust-targets.cmake")

# Check that all required components are available
check_required_components(lambdust)

# Set variables for compatibility
set(LAMBDUST_FOUND TRUE)
set(LAMBDUST_VERSION "0.1.1")
set(LAMBDUST_INCLUDE_DIRS "${CMAKE_CURRENT_LIST_DIR}/../../../include")

# Provide imported target
if(TARGET lambdust::lambdust AND NOT TARGET lambdust)
    add_library(lambdust ALIAS lambdust::lambdust)
endif()

# Print found message
if(NOT lambdust_FIND_QUIETLY)
    message(STATUS "Found Lambdust: ${LAMBDUST_VERSION}")
endif()
