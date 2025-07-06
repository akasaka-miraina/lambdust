# C++ Integration Examples
cmake_minimum_required(VERSION 3.15)

# Simple C++ wrapper example
add_executable(cpp_wrapper cpp_wrapper.cpp)
target_link_libraries(cpp_wrapper PRIVATE lambdust)

# Object-oriented integration example  
add_executable(cpp_oop_integration cpp_oop_integration.cpp)
target_link_libraries(cpp_oop_integration PRIVATE lambdust)

# RAII and modern C++ features example
add_executable(cpp_modern_features cpp_modern_features.cpp)
target_link_libraries(cpp_modern_features PRIVATE lambdust)

# Template-based integration example
add_executable(cpp_template_integration cpp_template_integration.cpp)
target_link_libraries(cpp_template_integration PRIVATE lambdust)

# Enhanced safety features example
add_executable(cpp_enhanced_safety cpp_enhanced_safety.cpp)
target_link_libraries(cpp_enhanced_safety PRIVATE lambdust)

# Game engine integration example
add_executable(cpp_game_engine cpp_game_engine.cpp)
target_link_libraries(cpp_game_engine PRIVATE lambdust)

# Copy Scheme files to build directory
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/scripts/game_logic.scm 
               ${CMAKE_CURRENT_BINARY_DIR}/game_logic.scm COPYONLY)
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/scripts/ai_behavior.scm 
               ${CMAKE_CURRENT_BINARY_DIR}/ai_behavior.scm COPYONLY)

# Set up testing
enable_testing()

add_test(NAME cpp_wrapper_test 
         COMMAND cpp_wrapper)
add_test(NAME cpp_oop_integration_test 
         COMMAND cpp_oop_integration)
add_test(NAME cpp_modern_features_test 
         COMMAND cpp_modern_features)
add_test(NAME cpp_template_integration_test 
         COMMAND cpp_template_integration)
add_test(NAME cpp_enhanced_safety_test 
         COMMAND cpp_enhanced_safety)
add_test(NAME cpp_game_engine_test 
         COMMAND cpp_game_engine)