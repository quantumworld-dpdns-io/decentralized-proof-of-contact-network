*** Settings ***
Resource    common.resource
Resource    api.resource

*** Variables ***
${ANALYTICS_QUERY_ENDPOINT}    ${API_BASE}/analytics/query
${ANALYTICS_EXPORT_ENDPOINT}    ${API_BASE}/analytics/export
${ANALYTICS_METRICS_ENDPOINT}    ${API_BASE}/analytics/metrics

*** Keywords ***
Run Analytics Query
    [Arguments]    ${query_name}    ${params}=${NONE}
    &{payload}=    Create Dictionary    name=${query_name}
    IF    ${params}    Set To Dictionary    ${payload}    params=${params}
    ${resp}=    API Post    ${ANALYTICS_QUERY_ENDPOINT}    payload=${payload}
    RETURN    ${resp.json()}

Export Analytics
    [Arguments]    ${query_name}    ${format}=json    ${params}=${NONE}
    &{payload}=    Create Dictionary    name=${query_name}    format=${format}
    IF    ${params}    Set To Dictionary    ${payload}    params=${params}
    ${resp}=    API Post    ${ANALYTICS_EXPORT_ENDPOINT}    payload=${payload}
    RETURN    ${resp.json()}

Get Proof Count Metrics
    [Arguments]    ${start_time}=${NONE}    ${end_time}=${NONE}
    &{payload}=    Create Dictionary
    IF    ${start_time}    Set To Dictionary    ${payload}    start_time=${start_time}
    IF    ${end_time}    Set To Dictionary    ${payload}    end_time=${end_time}
    ${resp}=    API Post    ${ANALYTICS_QUERY_ENDPOINT}    payload={"name": "proof_count_by_window", "params": ${payload}}
    RETURN    ${resp.json()}

Get Network Topology
    ${resp}=    API Post    ${ANALYTICS_QUERY_ENDPOINT}    payload={"name": "network_topology"}
    RETURN    ${resp.json()}

Get Verification Rate
    [Arguments]    ${window_id}
    ${resp}=    API Post    ${ANALYTICS_QUERY_ENDPOINT}    payload={"name": "verification_rate", "params": {"window_id": "${window_id}"}}
    RETURN    ${resp.json()}

Get Confidence Distribution
    ${resp}=    API Post    ${ANALYTICS_QUERY_ENDPOINT}    payload={"name": "confidence_score_distribution"}
    RETURN    ${resp.json()}

Export Proofs To Parquet
    [Arguments]    ${output_path}
    ${resp}=    API Post    ${ANALYTICS_EXPORT_ENDPOINT}    payload={"name": "all_proofs", "format": "parquet", "output": "${output_path}"}
    RETURN    ${resp.json()}

Export Proofs To CSV
    [Arguments]    ${output_path}
    ${resp}=    API Post    ${ANALYTICS_EXPORT_ENDPOINT}    payload={"name": "all_proofs", "format": "csv", "output": "${output_path}"}
    RETURN    ${resp.json()}

Query Should Return Data
    [Arguments]    ${query_name}
    ${data}=    Run Analytics Query    ${query_name}
    Should Not Be Empty    ${data}
    RETURN    ${data}

Verify Export File Exists
    [Arguments]    ${file_path}
    File Should Exist    ${file_path}
    ${size}=    Get File Size    ${file_path}
    Should Be True    ${size} > 0    Export file should not be empty
