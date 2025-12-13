<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;

class AstroController extends Controller
{
    public function events(Request $r)
    {
        $lat  = (float) $r->query('lat', 55.7558);
        $lon  = (float) $r->query('lon', 37.6176);
        $days = max(1, min(30, (int) $r->query('days', 7)));

        $from = now('UTC')->toDateString();
        $to   = now('UTC')->addDays($days)->toDateString();

        $appId  = env('ASTRO_APP_ID', '');
        $secret = env('ASTRO_APP_SECRET', '');
        if ($appId === '' || $secret === '') {
            return response()->json(['data' => [], 'message' => 'AstronomyAPI credentials not configured'], 200);
        }

        // AstronomyAPI использует Basic auth с appId:secret
        // Если API возвращает 403, возможно ключи неверные или истекли
        $auth = base64_encode($appId . ':' . $secret);
        
        $url  = 'https://api.astronomyapi.com/api/v2/bodies/events?' . http_build_query([
            'latitude'  => $lat,
            'longitude' => $lon,
            'from'      => $from,
            'to'        => $to,
        ]);

        $ch = curl_init($url);
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_HTTPHEADER     => [
                'Authorization: Basic ' . $auth,
                'Content-Type: application/json',
                'User-Agent: monolith-iss/1.0'
            ],
            CURLOPT_TIMEOUT        => 25,
        ]);
        $raw  = curl_exec($ch);
        $code = curl_getinfo($ch, CURLINFO_RESPONSE_CODE) ?: 0;
        $err  = curl_error($ch);
        curl_close($ch);

        if ($raw === false || $code >= 400) {
            // Возвращаем пустой результат вместо ошибки, чтобы не ломать интерфейс
            // Логируем ошибку для отладки
            error_log("AstronomyAPI error: " . ($err ?: "HTTP $code") . " - " . substr($raw, 0, 200));
            return response()->json([
                'data' => [],
                'message' => 'AstronomyAPI temporarily unavailable',
                'error' => $code >= 400 ? "HTTP $code" : null
            ]);
        }
        $json = json_decode($raw, true);
        return response()->json($json ?? ['data' => []]);
    }
}
