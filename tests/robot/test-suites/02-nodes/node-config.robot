*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/node.resource
Suite Setup    Setup Test Environment
Suite Teardown    Stop All Running Nodes
Test Tags    node    config    regression

*** Test Cases ***
Node Reports Config Via API
    [Tags]    smoke    node
    Start Node    node-config-1
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    Dictionary Should Contain Key    ${resp.json()}    node

Config Contains Node ID
    [Tags]    node
    Start Node    node-config-2
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    Should Be Equal    ${config}[node][id]    node-config-2

Config Contains Network Settings
    [Tags]    node
    Start Node    node-config-3
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${config}    network

Config Contains Storage Settings
    [Tags]    node
    Start Node    node-config-4
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${config}    storage

Config Contains API Settings
    [Tags]    node
    Start Node    node-config-5
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${config}    api

Config Reports Protocol Version
    [Tags]    node
    Start Node    node-config-6
    ${version}=    Get API Version
    Should Be Equal    ${version}[protocol_version]    ${PROTOCOL_VERSION}

Config Contains AI Settings
    [Tags]    node
    Start Node    node-config-7
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${config}    ai

Config Contains Observability Settings
    [Tags]    node
    Start Node    node-config-8
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${config}    observability

Config Values Are Valid Types
    [Tags]    node
    Start Node    node-config-9
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    ${node_cfg}=    Set Variable    ${config}[node]
    Should Be String    ${node_cfg}[id]
    Should Be String    ${node_cfg}[data_dir]
    Should Be String    ${node_cfg}[log_level]

Config Log Level Is Configurable
    [Tags]    node
    Start Node    node-config-10    port=9096
    ${resp}=    API Get    ${API_BASE}/api/v1/node/config
    ${config}=    Set Variable    ${resp.json()}
    Should Be Equal    ${config}[node][log_level]    debug
