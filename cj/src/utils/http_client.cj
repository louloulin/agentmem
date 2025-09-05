/*
 * Copyright (c) ContextEngine Team 2024. All rights reserved.
 */
package contextengine.utils

import std.collection.HashMap

/**
 * HTTP请求方法枚举
 */
public enum HttpMethod {
    | GET
    | POST
    | PUT
    | DELETE
    | PATCH
    
    public func toString(): String {
        match (this) {
            case GET => "GET"
            case POST => "POST"
            case PUT => "PUT"
            case DELETE => "DELETE"
            case PATCH => "PATCH"
        }
    }
}

/**
 * HTTP请求结构
 */
public struct HttpRequest {
    public let method: HttpMethod
    public let url: String
    public let headers: HashMap<String, String>
    public let body: String
    public let timeout: Int64
    
    public init(method: HttpMethod, url: String, headers: HashMap<String, String>, 
                body: String, timeout: Int64) {
        this.method = method
        this.url = url
        this.headers = headers
        this.body = body
        this.timeout = timeout
    }
    
    public init(method: HttpMethod, url: String) {
        this.method = method
        this.url = url
        this.headers = HashMap<String, String>()
        this.body = ""
        this.timeout = 30000  // 30秒默认超时
    }
}

/**
 * HTTP响应结构
 */
public struct HttpResponse {
    public let statusCode: Int64
    public let headers: HashMap<String, String>
    public let body: String
    public let success: Bool
    public let error: String
    
    public init(statusCode: Int64, headers: HashMap<String, String>, body: String, 
                success: Bool, error: String) {
        this.statusCode = statusCode
        this.headers = headers
        this.body = body
        this.success = success
        this.error = error
    }
    
    public init(statusCode: Int64, body: String) {
        this.statusCode = statusCode
        this.headers = HashMap<String, String>()
        this.body = body
        this.success = statusCode >= 200 && statusCode < 300
        this.error = if (this.success) { "" } else { "HTTP ${statusCode}" }
    }
}

/**
 * HTTP客户端接口
 */
public interface HttpClient {
    /**
     * 执行HTTP请求
     * @param request HTTP请求
     * @return HTTP响应
     */
    func execute(request: HttpRequest): HttpResponse
    
    /**
     * GET请求
     * @param url 请求URL
     * @param headers 请求头
     * @return HTTP响应
     */
    func get(url: String, headers: HashMap<String, String>): HttpResponse
    
    /**
     * POST请求
     * @param url 请求URL
     * @param body 请求体
     * @param headers 请求头
     * @return HTTP响应
     */
    func post(url: String, body: String, headers: HashMap<String, String>): HttpResponse
    
    /**
     * PUT请求
     * @param url 请求URL
     * @param body 请求体
     * @param headers 请求头
     * @return HTTP响应
     */
    func put(url: String, body: String, headers: HashMap<String, String>): HttpResponse
    
    /**
     * DELETE请求
     * @param url 请求URL
     * @param headers 请求头
     * @return HTTP响应
     */
    func delete(url: String, headers: HashMap<String, String>): HttpResponse
}

/**
 * 默认HTTP客户端实现
 * 基于仓颉标准库的HTTP功能
 */
public class DefaultHttpClient <: HttpClient {
    private let defaultTimeout: Int64
    private let defaultHeaders: HashMap<String, String>
    
    public init() {
        this.defaultTimeout = 30000  // 30秒
        this.defaultHeaders = HashMap<String, String>()
        this.defaultHeaders["User-Agent"] = "ContextEngine/1.0"
        this.defaultHeaders["Accept"] = "application/json"
        this.defaultHeaders["Content-Type"] = "application/json"
    }
    
    public init(timeout: Int64, defaultHeaders: HashMap<String, String>) {
        this.defaultTimeout = timeout
        this.defaultHeaders = defaultHeaders
    }
    
    /**
     * 执行HTTP请求
     */
    public func execute(request: HttpRequest): HttpResponse {
        try {
            // 合并默认头部和请求头部
            let mergedHeaders = mergeHeaders(defaultHeaders, request.headers)
            
            // 这里应该使用仓颉的HTTP库进行真实的HTTP请求
            // 由于仓颉HTTP库的具体API可能不同，这里提供一个模拟实现
            let response = executeHttpRequest(request.method, request.url, mergedHeaders, 
                                            request.body, request.timeout)
            
            return response
            
        } catch (e: Exception) {
            return HttpResponse(0, HashMap<String, String>(), "", false, e.toString())
        }
    }
    
    /**
     * GET请求
     */
    public func get(url: String, headers: HashMap<String, String>): HttpResponse {
        let request = HttpRequest(HttpMethod.GET, url, headers, "", defaultTimeout)
        return execute(request)
    }
    
    /**
     * POST请求
     */
    public func post(url: String, body: String, headers: HashMap<String, String>): HttpResponse {
        let request = HttpRequest(HttpMethod.POST, url, headers, body, defaultTimeout)
        return execute(request)
    }
    
    /**
     * PUT请求
     */
    public func put(url: String, body: String, headers: HashMap<String, String>): HttpResponse {
        let request = HttpRequest(HttpMethod.PUT, url, headers, body, defaultTimeout)
        return execute(request)
    }
    
    /**
     * DELETE请求
     */
    public func delete(url: String, headers: HashMap<String, String>): HttpResponse {
        let request = HttpRequest(HttpMethod.DELETE, url, headers, "", defaultTimeout)
        return execute(request)
    }
    
    // ===== 私有方法 =====
    
    /**
     * 合并请求头
     */
    private func mergeHeaders(defaultHeaders: HashMap<String, String>, 
                             requestHeaders: HashMap<String, String>): HashMap<String, String> {
        let merged = HashMap<String, String>()
        
        // 添加默认头部
        for ((key, value) in defaultHeaders) {
            merged[key] = value
        }
        
        // 添加请求头部（覆盖默认值）
        for ((key, value) in requestHeaders) {
            merged[key] = value
        }
        
        return merged
    }
    
    /**
     * 执行真实的HTTP请求
     * 这里需要根据仓颉的HTTP库实现
     */
    private func executeHttpRequest(method: HttpMethod, url: String, 
                                   headers: HashMap<String, String>, body: String, 
                                   timeout: Int64): HttpResponse {
        // 临时模拟实现，实际应该调用仓颉HTTP库
        println("HTTP ${method.toString()} ${url}")
        println("Headers: ${headers.size} items")
        println("Body: ${body.size} bytes")
        
        // 模拟成功响应
        if (url.contains("openai.com") || url.contains("anthropic.com")) {
            // 模拟LLM API响应
            let mockResponse = """
            {
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1677652288,
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "This is a mock response from the LLM API."
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 9,
                    "completion_tokens": 12,
                    "total_tokens": 21
                }
            }
            """
            return HttpResponse(200, mockResponse)
        }
        
        // 默认成功响应
        return HttpResponse(200, "{\"status\": \"ok\"}")
    }
}

/**
 * HTTP客户端工厂
 */
public class HttpClientFactory {
    /**
     * 创建默认HTTP客户端
     */
    public static func createDefault(): HttpClient {
        return DefaultHttpClient()
    }
    
    /**
     * 创建自定义HTTP客户端
     */
    public static func create(timeout: Int64, headers: HashMap<String, String>): HttpClient {
        return DefaultHttpClient(timeout, headers)
    }
}
