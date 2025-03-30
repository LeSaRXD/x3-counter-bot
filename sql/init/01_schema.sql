--
-- PostgreSQL database dump
--

-- Dumped from database version 17.4 (Ubuntu 17.4-1.pgdg24.04+2)
-- Dumped by pg_dump version 17.4 (Ubuntu 17.4-1.pgdg24.04+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: counter; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.counter (
    user_id bigint NOT NULL,
    server_id bigint NOT NULL,
    emote text NOT NULL,
    count integer DEFAULT 0 NOT NULL,
    CONSTRAINT counter_count_check CHECK ((count > 0))
);


--
-- Name: options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.options (
    user_id bigint NOT NULL,
    opt_out boolean DEFAULT false NOT NULL,
    silent integer,
    CONSTRAINT options_silent_check CHECK ((silent >= 0))
);


--
-- Name: server_options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.server_options (
    server_id bigint NOT NULL,
    mute_all integer,
    CONSTRAINT server_options_mute_all_check CHECK ((mute_all >= 0))
);


--
-- Name: counter counter_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.counter
    ADD CONSTRAINT counter_pkey PRIMARY KEY (user_id, server_id, emote);


--
-- PostgreSQL database dump complete
--

