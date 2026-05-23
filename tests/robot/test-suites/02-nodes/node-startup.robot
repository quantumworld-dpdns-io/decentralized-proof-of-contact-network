*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/node.resource
Suite Setup    Setup Test Environment
Suite Teardown    Stop All Running Nodes
Test Tags    node    startup    regression

*** Test Cases ***
Start Default Node
    [Tags]    smoke    node
    ${result}=    Start Node    node-startup-1
    Should Be Equal    ${result}[status]    running
    Node Should Be Running    node-startup-1

Node Process Has Valid PID
    [Tags]    node
    ${result}=    Start Node    node-startup-2
    Should Be True    ${result}[pid] > 0    PID should be positive
    Node Should Be Running    node-startup-2

Node Changes State To Active
    [Tags]    node
    ${result}=    Start Node    node-startup-3
    ${info}=    Get Node Status Info    node-startup-3
    Should Be Equal    ${info}[status]    running

Node Connects To API
    [Tags]    node
    ${result}=    Start Node    node-startup-4
    ${info}=    Get Node Info Via API
    Dictionary Should Contain Key    ${info}    node_id

Node Reports Initial State
    [Tags]    node
    ${result}=    Start Node    node-startup-5
    ${status}=    Get Node Status Via API
    Dictionary Should Contain Key    ${status}    state

Start Multiple Nodes
    [Tags]    node
    ${n1}=    Start Node    node-multi-1
    ${n2}=    Start Node    node-multi-2
    Node Should Be Running    node-multi-1
    Node Should Be Running    node-multi-2

Node Starts With Custom Config
    [Tags]    node
    ${result}=    Start Node    node-custom-config    port=9095
    Node Should Be Running    node-custom-config

Node Startup Time Is Acceptable
    [Tags]    node
    ${start}=    Evaluate    time.time()    time
    Start Node    node-startup-timing
    ${end}=    Evaluate    time.time()    time
    ${duration}=    Evaluate    ${end} - ${start}
    Should Be True    ${duration} < 30    Node should start within 30 seconds

Node Starts With Debug Logging
    [Tags]    node
    ${result}=    Start Node    node-debug-log
    Node Should Be Running    node-debug-log

Node API Reports Correct Node ID
    [Tags]    node
    ${result}=    Start Node    node-id-check
    ${info}=    Get Node Info Via API
    Should Be Equal    ${info}[node_id]    node-id-check
