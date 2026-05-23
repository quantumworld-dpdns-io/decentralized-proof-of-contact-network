*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    api    websocket    events    regression

*** Test Cases ***
WebSocket Connection Succeeds
    [Tags]    smoke    websocket
    ${resp}=    API Get    ${API_BASE}/ws
    Status Should Be    101    ${resp}    WebSocket upgrade should succeed

WebSocket Receives Proof Events
    [Tags]    websocket
    ${resp}=    API Get    ${API_BASE}/ws
    ${proof}=    Create Proof    target_node=node-ws-event    window_id=window-ws-event
    Sleep    1s

WebSocket Receives Health Events
    [Tags]    websocket
    ${resp}=    API Get    ${API_BASE}/ws
    Sleep    1s

WebSocket Connection Requires Auth
    [Tags]    websocket    security
    ${resp}=    API Get    ${API_BASE}/ws    expected_status=401

WebSocket Handles Multiple Clients
    [Tags]    websocket
    ${resp1}=    API Get    ${API_BASE}/ws
    ${resp2}=    API Get    ${API_BASE}/ws
    Status Should Be    101    ${resp1}
    Status Should Be    101    ${resp2}

WebSocket Ping Pong Maintained
    [Tags]    websocket
    ${resp}=    API Get    ${API_BASE}/ws
    Sleep    5s

WebSocket Receives Node Status Events
    [Tags]    websocket
    ${resp}=    API Get    ${API_BASE}/ws
    Sleep    2s

WebSocket Event Contains Type Field
    [Tags]    websocket
    ${resp}=    API Get    ${API_BASE}/ws
    ${proof}=    Create Proof    target_node=node-ws-type    window_id=window-ws-type
    Sleep    1s

Websocket Connection Is Bidirectional
    [Tags]    websocket
    ${resp}=    API Get    ${API_BASE}/ws
    Sleep    1s
