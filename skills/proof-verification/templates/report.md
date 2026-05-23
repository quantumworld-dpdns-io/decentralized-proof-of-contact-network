# Proof Verification Report

**Generated:** {{ generated_at }}
**Proof ID:** {{ proof_id }}

---

## Summary

**Status:** {{ status }}
**Confidence Score:** {{ confidence_score }}

## Verification Checks

| Check | Result | Detail |
|-------|--------|--------|
| Signature | {{ signature_valid }} | {{ signature_detail }} |
| Timestamp | {{ timestamp_valid }} | Drift: {{ timestamp_drift }}s |
| Orbital Window | {{ orbital_window_valid }} | {{ window_detail }} |
| Chain Integrity | {{ chain_integrity_valid }} | Position: {{ chain_position }} |

## Proof Details

| Field | Value |
|-------|-------|
| Proving Node | `{{ proving_node }}` |
| Target Node | `{{ target_node }}` |
| Window Start | {{ window_start }} |
| Window End | {{ window_end }} |
| Window Type | {{ window_type }} |
| Created At | {{ created_at }} |
| Protocol Version | {{ protocol_version }} |

## Raw Proof Data

```json
{{ proof_json }}
```

## Recommendations

{% if all_passed %}
- Proof is valid. No action needed.
{% else %}
- {% if not signature_valid %}Re-issue proof with correct keypair{% endif %}
- {% if not timestamp_valid %}Sync node clock via NTP{% endif %}
- {% if not orbital_window_valid %}Create proof in an active window{% endif %}
- {% if not chain_integrity_valid %}Rebuild chain from last known good position{% endif %}
{% endif %}
