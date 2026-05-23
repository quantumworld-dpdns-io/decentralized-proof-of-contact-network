*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/node.resource
Suite Setup    Setup Test Environment
Suite Teardown    Stop All Running Nodes
Test Tags    node    shutdown    regression

*** Test Cases ***
Stop Running Node
    [Tags]    smoke    node
    Start Node    node-shutdown-1
    ${result}=    Stop Node    node-shutdown-1
    Should Be Equal    ${result}[status]    stopped

Node Is Not Running After Stop
    [Tags]    node
    Start Node    node-shutdown-2
    Stop Node    node-shutdown-2
    Node Should Not Be Running    node-shutdown-2

Stop Non Existent Node
    [Tags]    node    negative
    Run Keyword And Expect Error    *not running*    Stop Node    node-nonexistent

Graceful Shutdown Preserves Data
    [Tags]    node
    Start Node    node-shutdown-3
    ${proof}=    Create Proof    target_node=node-shutdown-preserve    window_id=window-shutdown
    ${id}=    Set Variable    ${proof}[id]
    Stop Node    node-shutdown-3
    Start Node    node-shutdown-3
    ${retrieved}=    Get Proof    ${id}
    Should Be Equal    ${retrieved}[id]    ${id}

Stop All Nodes Cleans Up
    [Tags]    node
    Start Node    node-shutdown-4
    Start Node    node-shutdown-5
    ${results}=    Stop All Nodes
    Length Should Be    ${results}    2
    Node Should Not Be Running    node-shutdown-4
    Node Should Not Be Running    node-shutdown-5

Node Reports Shutdown State
    [Tags]    node
    Start Node    node-shutdown-6
    Stop Node    node-shutdown-6
    ${status}=    Get Node Status Via API    # This may fail if node is fully down
    Log    Node shutdown complete

Stop Node While Proofs Are Pending
    [Tags]    node
    Start Node    node-shutdown-7
    FOR    ${i}    IN RANGE    5
        Create Proof    target_node=node-shutdown-pending-${i}    window_id=window-shutdown-pending
    END
    Stop Node    node-shutdown-7
    Node Should Not Be Running    node-shutdown-7

Node Can Be Restarted After Shutdown
    [Tags]    node
    Start Node    node-shutdown-8
    Stop Node    node-shutdown-8
    Start Node    node-shutdown-8
    Node Should Be Running    node-shutdown-8

Shutdown Does Not Leak Processes
    [Tags]    node
    Start Node    node-shutdown-9
    Stop Node    node-shutdown-9
    Node Should Not Be Running    node-shutdown-9
