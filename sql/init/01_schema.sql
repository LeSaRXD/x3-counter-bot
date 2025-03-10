--
-- PostgreSQL database dump
--

-- Dumped from database version 14.17 (Ubuntu 14.17-0ubuntu0.22.04.1)
-- Dumped by pg_dump version 14.17 (Ubuntu 14.17-0ubuntu0.22.04.1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
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
    user_id text NOT NULL,
    server_id text NOT NULL,
    emote character varying(4) NOT NULL,
    count integer DEFAULT 0 NOT NULL,
    CONSTRAINT counter_count_check CHECK ((count > 0))
);


--
-- Name: options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.options (
    user_id text NOT NULL,
    opt_out boolean DEFAULT false NOT NULL,
    silent integer,
    CONSTRAINT options_silent_check CHECK ((silent >= 0))
);


--
-- Name: server_options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.server_options (
    server_id text NOT NULL,
    mute_all integer,
    CONSTRAINT server_options_mute_all_check CHECK ((mute_all >= 0))
);


--
-- Name: counter counter_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.counter
    ADD CONSTRAINT counter_pkey PRIMARY KEY (user_id, server_id, emote);


--
-- Name: options options_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.options
    ADD CONSTRAINT options_pkey PRIMARY KEY (user_id);


--
-- Name: server_options server_options_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.server_options
    ADD CONSTRAINT server_options_pkey PRIMARY KEY (server_id);


--
-- PostgreSQL database dump complete
--

