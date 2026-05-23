*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    smoke    version

*** Test Cases ***
Version Endpoint Returns Version
    ${data}=    Get API Version
    Dictionary Should Contain Key    ${data}    version

Version Format Is Semver
    ${data}=    Get API Version
    Should Match Regex    ${data}[version]    ^\\d+\\.\\d+\\.\\d+(-[a-zA-Z0-9.]+)?$

Version Returns Commit Hash
    ${data}=    Get API Version
    Dictionary Should Contain Key    ${data}    commit
    Should Match Regex    ${data}[commit]    ^[a-f0-9]{7,40}$

Version Returns Build Timestamp
    ${data}=    Get API Version
    Dictionary Should Contain Key    ${data}    build_time
    ${ts}=    Evaluate    __import__('datetime').datetime.fromisoformat('${data}[build_time]')
    Should Be True    ${ts} is not None

Version Returns Protocol Version
    ${data}=    Get API Version
    Dictionary Should Contain Key    ${data}    protocol_version
    Should Be Equal    ${data}[protocol_version]    ${PROTOCOL_VERSION}

Version Returns Rust Compiler Info
    ${data}=    Get API Version
    Dictionary Should Contain Key    ${data}    rust_version
    Should Match Regex    ${data}[rust_version]    ^\\d+\\.\\d+\\.\\d+

Version Returns Git Branch
    ${data}=    Get API Version
    Dictionary Should Contain Key    ${data}    git_branch

Version Returns Feature Flags
    ${data}=    Get API Version
    Dictionary Should Contain Key    ${data}    features
    Should Be True    ${data}[features] is not None

Version Endpoint Is Public
    ${resp}=    GET    ${API_BASE_URL}/version
    Status Should Be    200    ${resp}

Version Endpoint Is Stable
    ${data}=    Get API Version
    Should Be Equal    ${data}[version]    ${data}[version]
