/*
 * ZERO OS Native Bare-Metal Network Stack, Firewall, IDS/IPS & Security Subsystem
 * 0 Language Tier / Pure Standalone C
 */

#include <stdint.h>

#define FW_MAX_RULES 32
#define MAX_LOG_EVENTS 64

typedef struct {
    uint32_t src_ip;
    uint32_t dst_ip;
    uint16_t src_port;
    uint16_t dst_port;
    uint8_t  protocol; /* 1=ICMP, 6=TCP, 17=UDP */
    uint8_t  action;   /* 0=DENY, 1=ALLOW */
} ZeroFwRule;

typedef struct {
    ZeroFwRule rules[FW_MAX_RULES];
    uint32_t rule_count;
    uint32_t packets_allowed;
    uint32_t packets_blocked;
    uint32_t ids_alerts_triggered;
    uint32_t vpn_killswitch_active;
} ZeroNetSecurityEngine;

static ZeroNetSecurityEngine g_net_sec;

void zero_net_security_init(void) {
    g_net_sec.rule_count = 0;
    g_net_sec.packets_allowed = 0;
    g_net_sec.packets_blocked = 0;
    g_net_sec.ids_alerts_triggered = 0;
    g_net_sec.vpn_killswitch_active = 1; /* Mode VPN Kill-Switch actif par défaut */

    /* Règle Pare-feu 1 : Autoriser DNS (UDP 53) */
    ZeroFwRule *r1 = &g_net_sec.rules[0];
    r1->src_ip = 0; r1->dst_ip = 0; r1->src_port = 0; r1->dst_port = 53;
    r1->protocol = 17; r1->action = 1;

    /* Règle Pare-feu 2 : Autoriser HTTPS (TCP 443) */
    ZeroFwRule *r2 = &g_net_sec.rules[1];
    r2->src_ip = 0; r2->dst_ip = 0; r2->src_port = 0; r2->dst_port = 443;
    r2->protocol = 6; r2->action = 1;

    g_net_sec.rule_count = 2;
}

/* Inspection Filtrage Pare-feu & Détection d'Intrusion IDS/IPS */
uint8_t zero_net_inspect_packet(uint32_t src_ip, uint32_t dst_ip, uint16_t src_port, uint16_t dst_port, uint8_t protocol) {
    /* Détection IDS : Port Scan Attack (ex: tentative accès port SSH 22 ou Telnet 23 non autorisé) */
    if (dst_port == 22 || dst_port == 23) {
        g_net_sec.ids_alerts_triggered++;
        g_net_sec.packets_blocked++;
        return 0; /* IPS Block */
    }

    /* Traitement Pare-feu */
    for (uint32_t i = 0; i < g_net_sec.rule_count; i++) {
        ZeroFwRule *r = &g_net_sec.rules[i];
        if (r->protocol == protocol && (r->dst_port == 0 || r->dst_port == dst_port)) {
            if (r->action == 1) {
                g_net_sec.packets_allowed++;
                return 1; /* ALLOW */
            }
        }
    }

    g_net_sec.packets_blocked++;
    return 0; /* DENY par défaut */
}
