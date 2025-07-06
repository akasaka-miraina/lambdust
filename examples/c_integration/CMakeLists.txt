# C Integration Examples
cmake_minimum_required(VERSION 3.15)

# Basic example
add_executable(basic_usage basic_usage.c)
target_link_libraries(basic_usage PRIVATE lambdust)

# Host functions example
add_executable(host_functions host_functions.c)
target_link_libraries(host_functions PRIVATE lambdust)

# Calculator example
add_executable(calculator calculator.c)
target_link_libraries(calculator PRIVATE lambdust)

# Plugin system example
add_executable(plugin_system plugin_system.c)
target_link_libraries(plugin_system PRIVATE lambdust)

# Configuration example
add_executable(config_example config_example.c)
target_link_libraries(config_example PRIVATE lambdust)

# Copy Scheme files to build directory
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/scripts/math.scm 
               ${CMAKE_CURRENT_BINARY_DIR}/math.scm COPYONLY)
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/scripts/config.scm 
               ${CMAKE_CURRENT_BINARY_DIR}/config.scm COPYONLY)

# Set up testing
enable_testing()

add_test(NAME basic_usage_test 
         COMMAND basic_usage)
add_test(NAME host_functions_test 
         COMMAND host_functions)
add_test(NAME calculator_test 
         COMMAND calculator "(+ 2 3)")
add_test(NAME config_example_test 
         COMMAND config_example)