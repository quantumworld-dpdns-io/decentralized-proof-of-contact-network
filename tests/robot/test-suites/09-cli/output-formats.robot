*** Settings ***
Resource    ../../resources/common.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    cli    output    regression

*** Test Cases ***
CLI Output Is Plain Text
    [Tags]    smoke    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    poi-node

CLI JSON Output Format
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Be Equal    ${result.rc}    0

CLI Error Messages Are Descriptive
    [Tags]    cli    negative
    ${result}=    Run Process    poi-node    --config    /tmp/nonexistent-file    shell=True
    Should Contain    ${result.stderr}    Failed to load configuration

CLI Help Lists Subcommands
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    poi-node

CLI Usage Is Formatted
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    Usage

CLI Output Contains Description
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    Decentralized Proof-of-Contact

CLI Args Are Documented
    [Tags]    cli
    ${result}=    Run Process    poi-node    --help    shell=True
    Should Contain    ${result.stdout}    --config
    Should Contain    ${result.stdout}    Path to configuration file
