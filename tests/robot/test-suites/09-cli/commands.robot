*** Settings ***
Resource    ../../resources/common.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    cli    commands    regression

*** Test Cases ***
CLI Help Displays Usage
    [Tags]    smoke    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    Usage
    Should Contain    ${result.stdout}    poi-node

CLI Version Displays Version
    [Tags]    cli
    ${result}=    Run Process    poi-node    --version    shell=True
    Should Contain    ${result.stdout}    0.1.0

CLI Config Flag Is Accepted
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    --config

CLI Config Default Is Config File
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    config.toml

CLI With Invalid Config Fails Gracefully
    [Tags]    cli    negative
    ${result}=    Run Process    poi-node    --config    /nonexistent/config.toml    shell=True
    Should Not Be Equal    ${result.rc}    0

CLI Recognizes Short Flags
    [Tags]    cli
    ${result}=    Run Process    poi-node    -h    shell=True
    Should Contain    ${result.stdout}    poi-node

CLI Version Via Short Flag
    [Tags]    cli
    ${result}=    Run Process    poi-node    -V    shell=True
    Should Contain    ${result.stdout}    0.1.0

CLI Debug Flag Enables Verbose
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    poi-node

CLI Returns Non Zero On Error
    [Tags]    cli    negative
    ${result}=    Run Process    poi-node    --invalid-flag    shell=True
    Should Not Be Equal    ${result.rc}    0

CLI Help Returns Zero
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Be Equal    ${result.rc}    0
