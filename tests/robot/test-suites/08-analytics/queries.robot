*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/analytics.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    analytics    queries    regression

*** Test Cases ***
Proof Count By Window Query
    [Tags]    smoke    analytics
    Create Proof    target_node=node-analytics-count    window_id=window-analytics-count
    ${data}=    Run Analytics Query    proof_count_by_window
    Should Not Be Empty    ${data}

Network Topology Query
    [Tags]    analytics
    Create Proof    target_node=node-analytics-topo    window_id=window-analytics-topo
    ${data}=    Query Should Return Data    network_topology

Orbital Window Utilization
    [Tags]    analytics
    Create Proof    target_node=node-analytics-util    window_id=window-analytics-util
    ${data}=    Query Should Return Data    orbital_window_utilization

Confidence Score Distribution
    [Tags]    analytics
    Create Proof    target_node=node-analytics-conf    window_id=window-analytics-conf    confidence_score=0.9
    ${data}=    Query Should Return Data    confidence_score_distribution

Anomalous Patterns Query
    [Tags]    analytics
    Create Proof    target_node=node-analytics-anom    window_id=window-analytics-anom
    ${data}=    Run Analytics Query    anomalous_patterns
    Should Not Be None    ${data}

Verification Rate Query
    [Tags]    analytics
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    Create Proof    target_node=node-analytics-ver    window_id=${window}[id]
    ${data}=    Get Verification Rate    ${window}[id]
    Should Not Be Empty    ${data}

Node Activity Query
    [Tags]    analytics
    Create Proof    target_node=node-analytics-act    window_id=window-analytics-act
    ${data}=    Run Analytics Query    node_activity    params={"node_id": "${DEFAULT_NODE_ID}"}
    Should Not Be Empty    ${data}

Peer Reputation Trends
    [Tags]    analytics
    Create Proof    target_node=node-analytics-rep    window_id=window-analytics-rep
    ${data}=    Run Analytics Query    peer_reputation_trends    params={"peer_id": "${DEFAULT_NODE_ID}"}
    Should Not Be Empty    ${data}

Proof Chain Depth Distribution
    [Tags]    analytics
    Create Proof    target_node=node-analytics-depth-1    window_id=window-analytics-depth
    Create Proof    target_node=node-analytics-depth-2    window_id=window-analytics-depth
    ${data}=    Query Should Return Data    proof_chain_depth_distribution

All Predefined Queries Available
    [Tags]    analytics
    ${queries}=    Create List    proof_count_by_window    network_topology    orbital_window_utilization
    ...    confidence_score_distribution    anomalous_patterns
    FOR    ${name}    IN    @{queries}
        ${data}=    Run Analytics Query    ${name}
        Log    Query ${name} returned data
    END

Query With Invalid Name Fails
    [Tags]    analytics    negative
    Run Keyword And Expect Error    *404*    Run Analytics Query    non_existent_query

Query Respects Time Range
    [Tags]    analytics
    ${data}=    Run Analytics Query    proof_count_by_window    params={"start": "2025-01-01", "end": "2025-12-31"}
    Should Not Be None    ${data}
