vcpkg_from_github(
    OUT_SOURCE_PATH SOURCE_PATH
    REPO NLnetLabs/unbound
    REF release-1.23.0
    SHA512 760425c0d045c3b99952666256483fdd722c7181a355df87a160718835288ce1a9efcbb9bb2c9e1ef27b0991e3acb1e1ec0d256d370d4ec370daded26d723719
    HEAD_REF master
)

# Apply any necessary patches for Windows builds
# vcpkg_apply_patches(
#     SOURCE_PATH ${SOURCE_PATH}
#     PATCHES
#         fix-windows-build.patch
# )

# Set up build environment
if(VCPKG_TARGET_IS_WINDOWS)
    set(ENV{OPENSSL_ROOT_DIR} ${CURRENT_INSTALLED_DIR})
    set(ENV{OPENSSL_LIBDIR} ${CURRENT_INSTALLED_DIR}/lib)
    set(ENV{OPENSSL_INCLUDEDIR} ${CURRENT_INSTALLED_DIR}/include)
endif()

# Check for features
vcpkg_check_features(OUT_FEATURE_OPTIONS FEATURE_OPTIONS
    FEATURES
    libevent WITH_LIBEVENT
)

set(CONFIGURE_OPTIONS
    --with-ssl=${CURRENT_INSTALLED_DIR}
    --with-libexpat=${CURRENT_INSTALLED_DIR}
    --disable-shared
    --enable-static
    --disable-rpath
    --with-libunbound-only
)

if("libevent" IN_LIST FEATURES)
    list(APPEND CONFIGURE_OPTIONS --with-libevent=${CURRENT_INSTALLED_DIR})
else()
    list(APPEND CONFIGURE_OPTIONS --without-libevent)
endif()

if(VCPKG_TARGET_IS_WINDOWS)
    list(APPEND CONFIGURE_OPTIONS 
        --with-ssl-dir=${CURRENT_INSTALLED_DIR}
        ac_cv_func_getaddrinfo=yes
        ac_cv_func_getnameinfo=yes
    )
endif()

vcpkg_configure_make(
    SOURCE_PATH ${SOURCE_PATH}
    AUTOCONFIG
    OPTIONS 
        ${CONFIGURE_OPTIONS}
)

vcpkg_install_make()

# Clean up
vcpkg_fixup_pkgconfig()

# Remove debug binaries if not needed
if(EXISTS ${CURRENT_PACKAGES_DIR}/debug/bin)
    file(GLOB DEBUG_BINARIES ${CURRENT_PACKAGES_DIR}/debug/bin/*)
    if(DEBUG_BINARIES)
        file(REMOVE ${DEBUG_BINARIES})
    endif()
endif()

# Remove executables from release if we only want the library
if(EXISTS ${CURRENT_PACKAGES_DIR}/bin)
    file(GLOB RELEASE_BINARIES ${CURRENT_PACKAGES_DIR}/bin/unbound*)
    if(RELEASE_BINARIES)
        file(REMOVE ${RELEASE_BINARIES})
    endif()
endif()

# Install license
vcpkg_install_copyright(FILE_LIST ${SOURCE_PATH}/LICENSE)
